import Foundation

/// Whether a sent secret was opened (`GET /v1/status/:token`).
/// ≙ `DeliveryReceipt` in services-core, as the sender sees it.
public struct DeliveryStatus: Equatable, Sendable {
    public enum State: String, Sendable {
        /// Not opened yet, still within its lifetime.
        case waiting
        case opened
        /// Its lifetime ran out before anyone opened it.
        case expired
    }

    public let state: State
    public let expiresAt: Date
    /// The first reveal; peeking with `/meta` never counts.
    public let openedAt: Date?

    public init(state: State, expiresAt: Date, openedAt: Date?) {
        self.state = state
        self.expiresAt = expiresAt
        self.openedAt = openedAt
    }
}
