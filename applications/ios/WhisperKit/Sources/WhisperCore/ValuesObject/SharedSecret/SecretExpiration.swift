import Foundation

/// When a secret is deleted, opened or not: one of the composer's presets, or
/// a date of the user's choosing (the web form's "Custom..."). The server
/// refuses anything beyond seven days (`MAX_EXPIRATION_DAYS` in
/// `services-core/src/values_object/shared_secret/secret_expiration.rs`), so
/// that is the ceiling here too.
public enum SecretExpiration: Hashable, Sendable {
    case fiveMinutes
    case oneHour
    case oneDay
    case sevenDays
    case custom(Date)

    public static let presets: [SecretExpiration] = [.fiveMinutes, .oneHour, .oneDay, .sevenDays]
    public static let maximum: TimeInterval = 7 * 24 * 60 * 60

    public var label: String {
        switch self {
        case .fiveMinutes: return "5 minutes"
        case .oneHour: return "1 hour"
        case .oneDay: return "24 hours"
        case .sevenDays: return "7 days"
        case .custom(let date): return date.formatted(date: .abbreviated, time: .shortened)
        }
    }

    /// The compact form shown once a link exists.
    public var shortLabel: String {
        switch self {
        case .fiveMinutes: return "5m"
        case .oneHour: return "1h"
        case .oneDay: return "24h"
        case .sevenDays: return "7d"
        case .custom(let date): return date.formatted(date: .numeric, time: .shortened)
        }
    }

    public func date(from now: Date = Date()) -> Date {
        switch self {
        case .fiveMinutes: return now.addingTimeInterval(5 * 60)
        case .oneHour: return now.addingTimeInterval(60 * 60)
        case .oneDay: return now.addingTimeInterval(24 * 60 * 60)
        case .sevenDays: return now.addingTimeInterval(Self.maximum)
        case .custom(let date): return date
        }
    }

    /// Whether the server would accept it at `now`: in the future, within a week.
    public func isValid(at now: Date = Date()) -> Bool {
        let date = date(from: now)
        return date > now && date <= now.addingTimeInterval(Self.maximum)
    }
}
