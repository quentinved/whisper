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

    /// Where a sent secret stands for its sender. "Opened" means two different
    /// things: a self-destruct secret is gone for good, a reusable one can still
    /// be read until it expires. Mirrors `describe()` in `sent_secrets.js`.
    public enum Outcome: Equatable, Sendable {
        case waiting(expiresAt: Date)
        case openedAndDeleted(openedAt: Date?)
        case openedStillReadable(openedAt: Date?, until: Date)
        case openedThenExpired(openedAt: Date?)
        case expiredUnopened
    }

    public func outcome(selfDestruct: Bool, now: Date = Date()) -> Outcome {
        switch state {
        case .waiting: return .waiting(expiresAt: expiresAt)
        case .expired: return .expiredUnopened
        case .opened where selfDestruct: return .openedAndDeleted(openedAt: openedAt)
        case .opened where expiresAt > now:
            return .openedStillReadable(openedAt: openedAt, until: expiresAt)
        case .opened: return .openedThenExpired(openedAt: openedAt)
        }
    }
}
