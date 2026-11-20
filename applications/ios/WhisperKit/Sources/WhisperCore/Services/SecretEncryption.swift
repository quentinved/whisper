import Foundation

/// Port for client-side encryption. ≙ `SecretEncryption` in services-core;
/// implemented by `AesGcm` in WhisperCrypto.
public protocol SecretEncryption: Sendable {
    /// Seals under a fresh random key.
    func seal(_ plaintext: String) throws -> SealedSecret
    func open(_ payload: Data, with key: SecretKey) throws -> String
}

public enum SecretEncryptionError: LocalizedError, Equatable {
    case malformedPayload
    case wrongKey
    case notUTF8

    public var errorDescription: String? {
        switch self {
        case .malformedPayload:
            return "The server returned a payload this app can't read."
        case .wrongKey:
            return "This link's key doesn't match the secret. It may have been truncated when it was copied."
        case .notUTF8:
            return "The secret isn't valid text."
        }
    }
}
