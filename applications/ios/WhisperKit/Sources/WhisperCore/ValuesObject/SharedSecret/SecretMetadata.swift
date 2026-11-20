/// What the server is willing to say about a secret without consuming it
/// (`GET /secret/:id/meta`).
public struct SecretMetadata: Equatable, Sendable {
    public let exists: Bool
    public let clientEncrypted: Bool
    public let selfDestruct: Bool

    public init(exists: Bool, clientEncrypted: Bool, selfDestruct: Bool) {
        self.exists = exists
        self.clientEncrypted = clientEncrypted
        self.selfDestruct = selfDestruct
    }
}
