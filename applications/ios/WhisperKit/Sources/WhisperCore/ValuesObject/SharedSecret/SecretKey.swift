import Foundation

/// The 256-bit AES key that lives in a link's `#k=` fragment and never reaches
/// the server.
public struct SecretKey: Equatable, Sendable, CustomDebugStringConvertible {
    public static let byteCount = 32

    public let bytes: Data

    public init?(bytes: Data) {
        guard bytes.count == Self.byteCount else { return nil }
        self.bytes = bytes
    }

    public init?(base64URL: String) {
        guard let bytes = Base64URL.decode(base64URL) else { return nil }
        self.init(bytes: bytes)
    }

    public var base64URL: String { Base64URL.encode(bytes) }

    /// Keeps the key out of logs, `print` and crash reports.
    public var debugDescription: String { "SecretKey(redacted)" }
}
