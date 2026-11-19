import Foundation

/// The lifetimes offered in the composer. The server refuses anything beyond
/// seven days (`MAX_EXPIRATION_DAYS` in
/// `services-core/src/values_object/shared_secret/secret_expiration.rs`), so
/// that is the ceiling here too.
enum Expiration: String, CaseIterable, Identifiable {
    case fiveMinutes
    case oneHour
    case oneDay
    case sevenDays

    var id: String { rawValue }

    var label: String {
        switch self {
        case .fiveMinutes: return "5 minutes"
        case .oneHour: return "1 hour"
        case .oneDay: return "24 hours"
        case .sevenDays: return "7 days"
        }
    }

    /// The compact form shown once a link exists.
    var shortLabel: String {
        switch self {
        case .fiveMinutes: return "5m"
        case .oneHour: return "1h"
        case .oneDay: return "24h"
        case .sevenDays: return "7d"
        }
    }

    var duration: TimeInterval {
        switch self {
        case .fiveMinutes: return 5 * 60
        case .oneHour: return 60 * 60
        case .oneDay: return 24 * 60 * 60
        case .sevenDays: return 7 * 24 * 60 * 60
        }
    }

    func date(from now: Date = Date()) -> Date {
        now.addingTimeInterval(duration)
    }
}
