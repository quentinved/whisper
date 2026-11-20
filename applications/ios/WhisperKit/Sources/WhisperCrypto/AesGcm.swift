import CryptoKit
import Foundation
import WhisperCore

/// AES-256-GCM via CryptoKit. ≙ `AesGcm` in `adapters/aes-gcm-crypto`.
///
/// The wire payload is `nonce[12] ‖ ciphertext ‖ tag[16]`, which is exactly how
/// CryptoKit lays out `AES.GCM.SealedBox.combined` — so neither direction needs
/// manual framing. The same bytes are produced by `encrypt_ephemeral` in
/// `adapters/aes-gcm-crypto/src/lib.rs` and by `crypto.subtle` in
/// `applications/axum/server/assets/create_secret.js`, which is what makes a
/// link created here openable in a browser and vice versa.
public struct AesGcm: SecretEncryption {
    public init() {}

    public func seal(_ plaintext: String) throws -> SealedSecret {
        let key = SymmetricKey(size: .bits256)
        let box = try AES.GCM.seal(Data(plaintext.utf8), using: key)

        // `combined` is non-nil for the 12-byte nonces CryptoKit generates.
        guard let payload = box.combined,
            let secretKey = SecretKey(bytes: key.withUnsafeBytes { Data($0) })
        else { throw SecretEncryptionError.malformedPayload }

        return SealedSecret(key: secretKey, payload: payload)
    }

    public func open(_ payload: Data, with key: SecretKey) throws -> String {
        let box: AES.GCM.SealedBox
        do {
            box = try AES.GCM.SealedBox(combined: payload)
        } catch {
            throw SecretEncryptionError.malformedPayload
        }

        // A tag mismatch here means the wrong key or tampered bytes; GCM gives
        // us that check for free, so there is nothing further to verify.
        guard let plaintext = try? AES.GCM.open(box, using: SymmetricKey(data: key.bytes)) else {
            throw SecretEncryptionError.wrongKey
        }
        guard let text = String(data: plaintext, encoding: .utf8) else {
            throw SecretEncryptionError.notUTF8
        }
        return text
    }
}
