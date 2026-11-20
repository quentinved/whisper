import Foundation
import Testing
import WhisperCore
import WhisperCrypto

@Suite struct AesGcmTests {
    let crypto = AesGcm()

    @Test(arguments: ["hello", "unicode ✅ 🦀 复杂密码", "multi\nline\nsecret", String(repeating: "x", count: 65_536)])
    func roundTrips(plaintext: String) throws {
        let sealed = try crypto.seal(plaintext)
        #expect(try crypto.open(sealed.payload, with: sealed.key) == plaintext)
    }

    @Test func payloadIsNonceCiphertextTag() throws {
        let sealed = try crypto.seal("abc")
        #expect(sealed.payload.count == 12 + 3 + 16)
    }

    @Test func everySealUsesAFreshKey() throws {
        #expect(try crypto.seal("same").key != crypto.seal("same").key)
    }

    @Test func rejectsTheWrongKey() throws {
        let sealed = try crypto.seal("secret")
        let other = try crypto.seal("decoy").key
        #expect(throws: SecretEncryptionError.wrongKey) { try crypto.open(sealed.payload, with: other) }
    }

    @Test func rejectsTruncatedPayloads() throws {
        let sealed = try crypto.seal("secret")
        #expect(throws: SecretEncryptionError.malformedPayload) {
            try crypto.open(sealed.payload.prefix(10), with: sealed.key)
        }
    }

    /// Sealed by the Rust `encrypt_ephemeral`
    /// (`cargo run -p aes-gcm-crypto --example interop_vector -- encrypt …`).
    /// Pins the cross-language format without needing cargo at test time;
    /// `Tools/interop_check.sh` still covers the Swift → Rust direction.
    @Test func opensAPayloadSealedByRust() throws {
        let key = try #require(SecretKey(base64URL: "-QBcN_ge5phlCl_cm33ktnyY5IjzK3eNIY-fYZdQBME"))
        let payload = try #require(
            Base64URL.decode("YtkCvn6wZoMdce-vAJwQx4FXdTyObEK7eZ6evCb2NAaZWIYcZqWQHKLvjZ2Au4hw8fNPxdFmbEb3")
        )
        #expect(try crypto.open(payload, with: key) == "unicode ✅ 🦀 复杂密码")
    }
}
