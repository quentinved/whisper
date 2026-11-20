import Foundation

/// A secret as the server hands it back. ≙ `SharedSecret` in services-core,
/// seen from the client: either a legacy secret the server already decrypted,
/// or an opaque zero-knowledge payload only the link's key can open.
public struct SharedSecret: Equatable, Sendable {
    public enum Content: Equatable, Sendable {
        case plain(String)
        case clientEncrypted(Data)
    }

    public let content: Content
    public let selfDestruct: Bool

    public init(content: Content, selfDestruct: Bool) {
        self.content = content
        self.selfDestruct = selfDestruct
    }
}
