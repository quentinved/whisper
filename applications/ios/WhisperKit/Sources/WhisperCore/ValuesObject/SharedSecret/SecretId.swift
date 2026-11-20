import Foundation

/// A secret's UUID v4, as the server issues it. ≙ `SecretId` in services-core.
///
/// Kept as the server's own string rather than a `UUID` so links round-trip
/// byte-for-byte (Foundation would uppercase it).
public struct SecretId: Hashable, Sendable, CustomStringConvertible {
    public let rawValue: String

    public init?(_ raw: String) {
        guard UUID(uuidString: raw) != nil else { return nil }
        rawValue = raw
    }

    public var description: String { rawValue }
}
