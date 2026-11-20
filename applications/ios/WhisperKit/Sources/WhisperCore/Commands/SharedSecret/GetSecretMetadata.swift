/// Identifies a secret without consuming it. ≙ `GetSecretMetadata` in
/// services-core.
///
/// Opening a burn-after-reading secret is irreversible, so a tap that only
/// meant "I have a link" must stop here, before `GetSecretById`.
public struct GetSecretMetadata: Sendable {
    private let id: SecretId

    public init(id: SecretId) {
        self.id = id
    }

    /// Throws `.notFound` only when the server says so. Any other failure —
    /// offline, or a server older than `/meta` — answers `nil` ("unknown"), and
    /// the UI hedges its warning instead of promising.
    public func handle(repository: some SharedSecretRepository) async throws -> SecretMetadata? {
        let metadata = try? await repository.metadata(id: id)
        if let metadata, !metadata.exists {
            throw SharedSecretRepositoryError.notFound
        }
        return metadata
    }
}
