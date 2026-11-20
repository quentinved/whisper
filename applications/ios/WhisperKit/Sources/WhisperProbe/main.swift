// A macOS CLI over the exact adapters the app ships, for the cross-language
// and end-to-end scripts in applications/ios/Tools. ≙ the Rust
// `aes-gcm-crypto` example `interop_vector`.
//
//   whisper-probe encrypt <plaintext>          -> "<key> <payload>"
//   whisper-probe decrypt <key> <payload>      -> plaintext
//   whisper-probe create <server> <secret> [burn]
//   whisper-probe open <link>
//   whisper-probe payload <link>               -> "<key> <stored payload>"
import Foundation
import WhisperCore
import WhisperCrypto
import WhisperHTTP

struct ProbeError: LocalizedError {
    let errorDescription: String?
}

let args = CommandLine.arguments
let encryption = AesGcm()

func link(_ raw: String) throws -> WhisperLink {
    guard let link = WhisperLink(parsing: raw) else { throw ProbeError(errorDescription: "unparseable link") }
    return link
}

func run() async throws {
    switch args.count > 1 ? args[1] : "" {
    case "encrypt":
        let sealed = try encryption.seal(args[2])
        print("\(sealed.key.base64URL) \(Base64URL.encode(sealed.payload))")

    case "decrypt":
        guard let key = SecretKey(base64URL: args[2]), let payload = Base64URL.decode(args[3]) else {
            throw ProbeError(errorDescription: "malformed key or payload")
        }
        print(try encryption.open(payload, with: key))

    case "create":
        let repository = HTTPSharedSecretRepository(server: URL(string: args[2])!)
        let command = try CreateClientEncryptedSecret(
            plaintext: args[3],
            expiration: .oneHour,
            selfDestruct: args.count > 4 && args[4] == "burn"
        )
        print(try await command.handle(encryption: encryption, repository: repository).link.url.absoluteString)

    case "open":
        let link = try link(args[2])
        let revealed = try await GetSecretById(link: link)
            .handle(encryption: encryption, repository: HTTPSharedSecretRepository(server: link.server))
        print(revealed.text)

    case "payload":
        // The raw stored bytes plus the key, so the Rust side can try them.
        let link = try link(args[2])
        let secret = try await HTTPSharedSecretRepository(server: link.server).fetch(id: link.id)
        guard case .clientEncrypted(let payload) = secret.content else {
            throw ProbeError(errorDescription: "not a client-encrypted secret")
        }
        print("\(link.key.base64URL) \(Base64URL.encode(payload))")

    default:
        throw ProbeError(errorDescription: "usage: whisper-probe <encrypt|decrypt|create|open|payload> …")
    }
}

do {
    try await run()
} catch {
    FileHandle.standardError.write(Data("\(error.localizedDescription)\n".utf8))
    exit(1)
}
