import Foundation

public enum CreateClientEncryptedSecretError: LocalizedError, Equatable {
    case empty
    case tooLarge(bytes: Int)
    case invalidExpiration

    public var errorDescription: String? {
        switch self {
        case .empty: return "Type a secret first."
        case .tooLarge: return "That secret is too large. The limit is 64 KB."
        case .invalidExpiration: return "Pick an expiry in the future, at most 7 days away."
        }
    }
}

/// A stored secret: the link to share, and the token that lets only the sender
/// see whether it was opened (`nil` on servers without delivery status).
public struct CreatedSecret: Sendable {
    public let link: WhisperLink
    public let statusToken: StatusToken?
}

/// Seals a secret on the device and stores only the ciphertext.
/// ≙ `CreateClientEncryptedSecret` in services-core, from the sender's side.
public struct CreateClientEncryptedSecret: Sendable {
    /// Matches `MAX_SECRET_SIZE` in
    /// `services-core/src/commands/shared_secret/create_secret.rs`. Measured in
    /// bytes, not characters, because that is what the server counts.
    public static let maxBytes = 64 * 1024

    private let plaintext: String
    private let expiration: SecretExpiration
    private let selfDestruct: Bool

    public init(plaintext: String, expiration: SecretExpiration, selfDestruct: Bool) throws {
        guard !plaintext.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
            throw CreateClientEncryptedSecretError.empty
        }
        let bytes = plaintext.utf8.count
        guard bytes <= Self.maxBytes else {
            throw CreateClientEncryptedSecretError.tooLarge(bytes: bytes)
        }
        self.plaintext = plaintext
        self.expiration = expiration
        self.selfDestruct = selfDestruct
    }

    /// Sealing happens before anything touches the network, so the plaintext
    /// never leaves this process and the key never leaves the device.
    public func handle(
        encryption: some SecretEncryption,
        repository: some SharedSecretRepository,
        now: Date = Date()
    ) async throws -> CreatedSecret {
        // Checked here rather than in `init`: a custom date that was fine when
        // chosen can lapse while the composer sits open.
        guard expiration.isValid(at: now) else {
            throw CreateClientEncryptedSecretError.invalidExpiration
        }
        let sealed = try encryption.seal(plaintext)
        let saved = try await repository.save(
            payload: sealed.payload,
            expiration: expiration.date(from: now),
            selfDestruct: selfDestruct
        )
        return CreatedSecret(
            link: WhisperLink(id: saved.id, key: sealed.key, server: repository.server),
            statusToken: saved.statusToken
        )
    }
}
