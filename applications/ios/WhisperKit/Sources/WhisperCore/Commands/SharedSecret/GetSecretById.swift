/// The plaintext of an opened secret, and whether opening it destroyed it.
public struct RevealedSecret: Equatable, Sendable {
    public let text: String
    public let burned: Bool
}

/// Consumes a secret and decrypts it with the key from its link.
/// ≙ `GetSecretById` in services-core. Irreversible for a self-destruct secret.
public struct GetSecretById: Sendable {
    private let link: WhisperLink

    public init(link: WhisperLink) {
        self.link = link
    }

    public func handle(
        encryption: some SecretEncryption,
        repository: some SharedSecretRepository
    ) async throws -> RevealedSecret {
        let secret = try await repository.fetch(id: link.id)

        // A secret created before zero-knowledge sharing existed comes back
        // already decrypted by the server; a modern one needs the link's key.
        let text: String
        switch secret.content {
        case .plain(let plaintext):
            text = plaintext
        case .clientEncrypted(let payload):
            text = try encryption.open(payload, with: link.key)
        }
        return RevealedSecret(text: text, burned: secret.selfDestruct)
    }
}
