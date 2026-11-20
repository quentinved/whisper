import Foundation

/// Sender-only capability to read a secret's delivery status. It can never
/// open, decrypt or burn the secret it tracks, and the share link cannot read
/// the status. ≙ `StatusToken` in services-core (a UUID v4).
///
/// Codable so the Sent list can keep it on the device; decoding validates it
/// like `init?` does.
public struct StatusToken: Hashable, Sendable, Codable, CustomStringConvertible {
    public let rawValue: String

    public init?(_ raw: String) {
        guard UUID(uuidString: raw) != nil else { return nil }
        rawValue = raw
    }

    public init(from decoder: any Decoder) throws {
        let raw = try decoder.singleValueContainer().decode(String.self)
        guard let token = StatusToken(raw) else {
            throw DecodingError.dataCorrupted(
                .init(codingPath: decoder.codingPath, debugDescription: "not a status token"))
        }
        self = token
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(rawValue)
    }

    public var description: String { rawValue }
}
