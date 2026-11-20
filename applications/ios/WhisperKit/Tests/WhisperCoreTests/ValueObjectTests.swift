import Foundation
import Testing
import WhisperCore

@Suite struct WhisperLinkTests {
    static let id = "3f2b8c1e-9d4a-4b7e-a1c2-5e6f7a8b9c0d"
    static let key = Base64URL.encode(Data(repeating: 0xAB, count: 32))
    static let fallback = URL(string: "https://configured.example")!

    @Test(arguments: [
        // CLI / Slack / Discord / this app
        "https://whisper.example.com/get_secret?shared_secret_id=\(id)#k=\(key)",
        // the web UI
        "https://whisper.example.com/?shared_secret_id=\(id)#k=\(key)",
        // pasted from a messaging app
        "  <https://whisper.example.com/get_secret?shared_secret_id=\(id)#k=\(key)>\n",
    ])
    func parsesLinksFromEveryClient(raw: String) throws {
        let link = try #require(WhisperLink(parsing: raw, fallbackServer: Self.fallback))
        #expect(link.id.rawValue == Self.id)
        #expect(link.key.base64URL == Self.key)
        #expect(link.server == URL(string: "https://whisper.example.com"))
    }

    @Test func keepsASelfHostedPort() throws {
        let link = try #require(
            WhisperLink(parsing: "http://localhost:1212/get_secret?shared_secret_id=\(Self.id)#k=\(Self.key)")
        )
        #expect(link.server == URL(string: "http://localhost:1212"))
    }

    @Test func deepLinksUseTheConfiguredServer() throws {
        let raw = "whisper://open?shared_secret_id=\(Self.id)#k=\(Self.key)"
        #expect(WhisperLink(parsing: raw) == nil)
        #expect(WhisperLink(parsing: raw, fallbackServer: Self.fallback)?.server == Self.fallback)
    }

    @Test(arguments: [
        "https://whisper.example.com/get_secret?shared_secret_id=\(id)",  // no key
        "https://whisper.example.com/get_secret?shared_secret_id=\(id)#k=",  // empty key
        "https://whisper.example.com/get_secret?shared_secret_id=\(id)#k=AAAA",  // short key
        "https://whisper.example.com/get_secret?shared_secret_id=\(id)#key=\(key)",  // wrong fragment
        "https://whisper.example.com/get_secret?shared_secret_id=nope#k=\(key)",  // not a UUID
        "ftp://whisper.example.com/get_secret?shared_secret_id=\(id)#k=\(key)",  // unknown scheme
        "not a link",
    ])
    func rejectsLinksThatCannotDecrypt(raw: String) {
        #expect(WhisperLink(parsing: raw, fallbackServer: Self.fallback) == nil)
    }

    @Test func canonicalURLRoundTrips() throws {
        let link = try #require(
            WhisperLink(parsing: "https://whisper.example.com/?shared_secret_id=\(Self.id)#k=\(Self.key)")
        )
        #expect(
            link.url.absoluteString
                == "https://whisper.example.com/get_secret?shared_secret_id=\(Self.id)#k=\(Self.key)"
        )
        #expect(WhisperLink(parsing: link.url.absoluteString) == link)
    }

    @Test func neverPrintsTheKey() throws {
        let link = try #require(
            WhisperLink(parsing: "https://whisper.example.com/?shared_secret_id=\(Self.id)#k=\(Self.key)")
        )
        #expect(!String(describing: link).contains(Self.key))
        #expect(!String(reflecting: link).contains(Self.key))
    }
}

@Suite struct Base64URLTests {
    @Test func roundTripsWithoutPaddingOrStandardAlphabet() {
        let data = Data([0xFB, 0xFF, 0xBF, 0x00, 0x01])
        let encoded = Base64URL.encode(data)
        #expect(!encoded.contains("=") && !encoded.contains("+") && !encoded.contains("/"))
        #expect(Base64URL.decode(encoded) == data)
    }

    @Test func rejectsImpossibleLengths() {
        #expect(Base64URL.decode("A") == nil)
        #expect(Base64URL.decode("AAAAA") == nil)
    }
}

@Suite struct SecretIdTests {
    @Test func acceptsOnlyUUIDs() {
        #expect(SecretId("3f2b8c1e-9d4a-4b7e-a1c2-5e6f7a8b9c0d")?.rawValue == "3f2b8c1e-9d4a-4b7e-a1c2-5e6f7a8b9c0d")
        #expect(SecretId("../../etc/passwd") == nil)
    }
}
