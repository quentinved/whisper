import CryptoKit
import Foundation

/// Client-side AES-256-GCM for Whisper's zero-knowledge ephemeral secrets.
///
/// The wire payload is `nonce[12] ‖ ciphertext ‖ tag[16]`, which is exactly how
/// CryptoKit lays out `AES.GCM.SealedBox.combined` — so neither direction needs
/// manual framing. The same bytes are produced by `encrypt_ephemeral` in
/// `adapters/aes-gcm-crypto/src/lib.rs` and by `crypto.subtle` in
/// `applications/axum/server/assets/create_secret.js`, which is what makes a
/// link created here openable in a browser and vice versa.
///
/// `Tools/interop_check.sh` asserts that round-trip against the Rust crate.
enum EphemeralCrypto {
    enum Failure: LocalizedError, Equatable {
        case malformedKey
        case malformedPayload
        case wrongKey
        case notUTF8

        var errorDescription: String? {
            switch self {
            case .malformedKey:
                return "This link's key is malformed."
            case .malformedPayload:
                return "The server returned a payload this app can't read."
            case .wrongKey:
                return "This link's key doesn't match the secret. It may have been truncated when it was copied."
            case .notUTF8:
                return "The secret isn't valid text."
            }
        }
    }

    /// A sealed secret, split into the half that goes to the server and the
    /// half that never does.
    struct Sealed {
        /// Travels only in the URL fragment. Browsers never send a fragment to
        /// the server, and neither do we.
        let keyB64URL: String
        /// Opaque bytes for the server to store. Useless without the key.
        let payloadB64URL: String
    }

    static func seal(_ plaintext: String) throws -> Sealed {
        let key = SymmetricKey(size: .bits256)
        let box = try AES.GCM.seal(Data(plaintext.utf8), using: key)

        // `combined` is non-nil for the 12-byte nonces CryptoKit generates.
        guard let combined = box.combined else { throw Failure.malformedPayload }

        return Sealed(
            keyB64URL: Base64URL.encode(key.withUnsafeBytes { Data($0) }),
            payloadB64URL: Base64URL.encode(combined)
        )
    }

    static func open(payloadB64URL: String, keyB64URL: String) throws -> String {
        guard let keyBytes = Base64URL.decode(keyB64URL), keyBytes.count == 32 else {
            throw Failure.malformedKey
        }
        guard let payload = Base64URL.decode(payloadB64URL) else {
            throw Failure.malformedPayload
        }

        let box: AES.GCM.SealedBox
        do {
            box = try AES.GCM.SealedBox(combined: payload)
        } catch {
            throw Failure.malformedPayload
        }

        // A tag mismatch here means the wrong key or tampered bytes; GCM gives
        // us that check for free, so there is nothing further to verify.
        guard let plaintext = try? AES.GCM.open(box, using: SymmetricKey(data: keyBytes)) else {
            throw Failure.wrongKey
        }
        guard let text = String(data: plaintext, encoding: .utf8) else {
            throw Failure.notUTF8
        }
        return text
    }
}
