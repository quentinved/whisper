import Foundation

/// Port to wherever secrets are stored — a Whisper server, from the app's point
/// of view. ≙ `SharedSecretRepository` in services-core; implemented by
/// `HTTPSharedSecretRepository` in WhisperHTTP.
public protocol SharedSecretRepository: Sendable {
    /// The origin share links should point at.
    var server: URL { get }

    /// Stores a client-encrypted payload with delivery tracking, and returns
    /// its id plus — on servers that track delivery — a status token.
    func save(payload: Data, expiration: Date, selfDestruct: Bool) async throws -> SavedSecret

    /// Non-consuming peek. `nil` means "unknown" (e.g. a server predating
    /// `/meta`), never "safe".
    func metadata(id: SecretId) async throws -> SecretMetadata?

    /// Consuming read. For a self-destruct secret this is irreversible.
    func fetch(id: SecretId) async throws -> SharedSecret

    /// Sender-only, non-consuming. `nil` when the server doesn't know the token.
    func deliveryStatus(token: StatusToken) async throws -> DeliveryStatus?
}

/// What `save` hands back. `statusToken` is absent on servers that predate
/// delivery status (before v1.5.0): the secret is then simply not tracked.
public struct SavedSecret: Equatable, Sendable {
    public let id: SecretId
    public let statusToken: StatusToken?

    public init(id: SecretId, statusToken: StatusToken?) {
        self.id = id
        self.statusToken = statusToken
    }
}

public enum SharedSecretRepositoryError: LocalizedError, Equatable {
    case notFound
    case tooLarge
    case offline
    case server(Int)
    case malformedResponse

    public var errorDescription: String? {
        switch self {
        case .notFound:
            return "This secret no longer exists. It may have expired, or someone opened it first."
        case .tooLarge:
            return "That secret is too large. The limit is 64 KB."
        case .offline:
            return "Can't reach the server. Check your connection."
        case .server(let code):
            return "The server returned an error (\(code))."
        case .malformedResponse:
            return "The server sent a response this app couldn't read."
        }
    }
}
