import Foundation

public enum StructuredSecretError: LocalizedError, Equatable {
    case empty
    case duplicateKey(String)
    case unnamedSection
    case duplicateName(String)
    case duplicateKeyInSection(section: String, key: String)

    // Same wording as the web form (`assets/create_secret.js`).
    public var errorDescription: String? {
        switch self {
        case .empty:
            return "Add at least one filled key/value pair."
        case .duplicateKey(let key):
            return "Duplicate key: \"\(key)\". Each key must be unique."
        case .unnamedSection:
            return "Section name is required (or remove the empty section)."
        case .duplicateName(let name):
            return "Duplicate name: \"\(name)\". Section names cannot collide with top-level keys or other sections."
        case .duplicateKeyInSection(let section, let key):
            return "Duplicate key in section \"\(section)\": \"\(key)\"."
        }
    }
}

/// A secret made of named values, optionally grouped into sections — the web's
/// "Multiple Values" mode. It travels as a plain JSON object string, so every
/// client (CLI, Slack, an older app) still sees a readable secret.
public struct StructuredSecret: Equatable, Sendable {
    public enum Entry: Equatable, Sendable {
        /// A string value.
        case value(key: String, value: String)
        /// A number, boolean, null or array from hand-written JSON, kept as written.
        case literal(key: String, json: String)
        case section(name: String, entries: [Entry])
    }

    /// What the composer collects: rows of key/value, and named groups of rows.
    public struct Row: Sendable {
        public var key: String
        public var value: String
        public init(key: String, value: String) {
            self.key = key
            self.value = value
        }
    }

    public struct Section: Sendable {
        public var name: String
        public var rows: [Row]
        public init(name: String, rows: [Row]) {
            self.name = name
            self.rows = rows
        }
    }

    public let entries: [Entry]

    /// Builds from the composer with the web form's rules: rows missing a key
    /// or a value are skipped, keys and section names are trimmed, empty
    /// sections are dropped, and names must be unique at each level.
    public init(rows: [Row], sections: [Section]) throws {
        var entries = try Self.collect(rows) { .duplicateKey($0) }
        var names = Set(entries.map(\.name))

        for section in sections {
            let name = section.name.trimmingCharacters(in: .whitespacesAndNewlines)
            let inner = try Self.collect(section.rows) {
                .duplicateKeyInSection(section: name, key: $0)
            }
            if inner.isEmpty { continue }
            guard !name.isEmpty else { throw StructuredSecretError.unnamedSection }
            guard names.insert(name).inserted else { throw StructuredSecretError.duplicateName(name) }
            entries.append(.section(name: name, entries: inner))
        }

        guard !entries.isEmpty else { throw StructuredSecretError.empty }
        self.entries = entries
    }

    /// Recognises a structured secret the way the web reveal page does: the
    /// text, trimmed, is a non-empty JSON object. Anything else is plain text.
    public init?(parsing text: String) {
        let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
        guard trimmed.hasPrefix("{"), trimmed.hasSuffix("}"),
            (try? JSONSerialization.jsonObject(with: Data(trimmed.utf8))) is [String: Any],
            case .object(let members)? = OrderedJSON.parse(trimmed),
            !members.isEmpty
        else { return nil }
        entries = Self.entries(from: members)
    }

    /// Compact JSON in entry order, as the web form's `JSON.stringify` sends it.
    public var json: String { Self.render(entries, indent: nil) }

    /// Two-space indented JSON for the raw view.
    public var prettyJSON: String { Self.render(entries, indent: "") }

    // MARK: - Building

    private static func collect(
        _ rows: [Row], duplicate: (String) -> StructuredSecretError
    ) throws -> [Entry] {
        var seen = Set<String>()
        var entries: [Entry] = []
        for row in rows {
            let key = row.key.trimmingCharacters(in: .whitespacesAndNewlines)
            guard !key.isEmpty, !row.value.isEmpty else { continue }
            guard seen.insert(key).inserted else { throw duplicate(key) }
            entries.append(.value(key: key, value: row.value))
        }
        return entries
    }

    private static func entries(from members: [(String, OrderedJSON.Value)]) -> [Entry] {
        members.map { key, value in
            switch value {
            case .object(let inner): return .section(name: key, entries: entries(from: inner))
            case .string(let text): return .value(key: key, value: text)
            case .raw(let json): return .literal(key: key, json: json)
            }
        }
    }

    // MARK: - Rendering

    private static func quoted(_ text: String) -> String {
        let encoder = JSONEncoder()
        encoder.outputFormatting = .withoutEscapingSlashes
        return String(decoding: (try? encoder.encode(text)) ?? Data("\"\"".utf8), as: UTF8.self)
    }

    /// `indent` is `nil` for compact output, else the current line's indent.
    private static func render(_ entries: [Entry], indent: String?) -> String {
        guard !entries.isEmpty else { return "{}" }
        let inner = indent.map { $0 + "  " }
        let members = entries.map { entry -> String in
            let value: String
            switch entry {
            case .value(_, let text): value = quoted(text)
            case .literal(_, let json): value = json
            case .section(_, let nested): value = render(nested, indent: inner)
            }
            return (inner ?? "") + quoted(entry.name) + (indent == nil ? ":" : ": ") + value
        }
        guard let indent else { return "{" + members.joined(separator: ",") + "}" }
        return "{\n" + members.joined(separator: ",\n") + "\n" + indent + "}"
    }
}

extension StructuredSecret.Entry {
    public var name: String {
        switch self {
        case .value(let key, _), .literal(let key, _): return key
        case .section(let name, _): return name
        }
    }
}

/// Just enough JSON to read an object while keeping its key order, which
/// Foundation's parsers drop. Only ever fed text `JSONSerialization` already
/// accepted, so it doesn't need to diagnose malformed input — just not crash.
private struct OrderedJSON {
    indirect enum Value {
        case object([(String, Value)])
        case string(String)
        /// Numbers, booleans, null and arrays, as the original text.
        case raw(String)
    }

    private let bytes: [UInt8]
    private var index = 0

    static func parse(_ text: String) -> Value? {
        var parser = OrderedJSON(bytes: Array(text.utf8))
        return parser.value()
    }

    private init(bytes: [UInt8]) {
        self.bytes = bytes
    }

    private var current: UInt8? { index < bytes.count ? bytes[index] : nil }

    private mutating func skipWhitespace() {
        while let byte = current, byte == 0x20 || byte == 0x0A || byte == 0x0D || byte == 0x09 {
            index += 1
        }
    }

    private mutating func value() -> Value? {
        skipWhitespace()
        switch current {
        case UInt8(ascii: "{"): return object()
        case UInt8(ascii: "\""): return string().map(Value.string)
        case nil: return nil
        default:
            let start = index
            guard skipRawValue() else { return nil }
            return .raw(String(decoding: bytes[start..<index], as: UTF8.self))
        }
    }

    private mutating func object() -> Value? {
        index += 1  // {
        var members: [(String, Value)] = []
        skipWhitespace()
        if current == UInt8(ascii: "}") {
            index += 1
            return .object(members)
        }
        while true {
            skipWhitespace()
            guard let key = string() else { return nil }
            skipWhitespace()
            guard current == UInt8(ascii: ":") else { return nil }
            index += 1
            guard let value = value() else { return nil }
            // Like `JSON.parse`: a repeated key keeps its first position, last value.
            if let existing = members.firstIndex(where: { $0.0 == key }) {
                members[existing].1 = value
            } else {
                members.append((key, value))
            }
            skipWhitespace()
            switch current {
            case UInt8(ascii: ","): index += 1
            case UInt8(ascii: "}"):
                index += 1
                return .object(members)
            default: return nil
            }
        }
    }

    private mutating func string() -> String? {
        guard current == UInt8(ascii: "\"") else { return nil }
        index += 1
        var out: [UInt8] = []
        while let byte = current {
            index += 1
            switch byte {
            case UInt8(ascii: "\""):
                return String(decoding: out, as: UTF8.self)
            case UInt8(ascii: "\\"):
                guard let escaped = current else { return nil }
                index += 1
                switch escaped {
                case UInt8(ascii: "b"): out.append(0x08)
                case UInt8(ascii: "f"): out.append(0x0C)
                case UInt8(ascii: "n"): out.append(0x0A)
                case UInt8(ascii: "r"): out.append(0x0D)
                case UInt8(ascii: "t"): out.append(0x09)
                case UInt8(ascii: "u"):
                    guard let scalar = unicodeEscape() else { return nil }
                    out.append(contentsOf: Array(String(Character(scalar)).utf8))
                default: out.append(escaped)  // \" \\ \/
                }
            default:
                out.append(byte)
            }
        }
        return nil
    }

    /// `\uXXXX`, joining a UTF-16 surrogate pair into one scalar.
    private mutating func unicodeEscape() -> Unicode.Scalar? {
        guard let high = hex4() else { return nil }
        if (0xD800...0xDBFF).contains(high),
            current == UInt8(ascii: "\\"), index + 1 < bytes.count,
            bytes[index + 1] == UInt8(ascii: "u")
        {
            index += 2
            guard let low = hex4(), (0xDC00...0xDFFF).contains(low) else { return nil }
            return Unicode.Scalar(0x10000 + ((high - 0xD800) << 10) + (low - 0xDC00))
        }
        return Unicode.Scalar(high) ?? Unicode.Scalar(0xFFFD)
    }

    private mutating func hex4() -> UInt32? {
        guard index + 4 <= bytes.count,
            let value = UInt32(String(decoding: bytes[index..<index + 4], as: UTF8.self), radix: 16)
        else { return nil }
        index += 4
        return value
    }

    /// Steps over a number, literal or array (strings inside arrays included).
    private mutating func skipRawValue() -> Bool {
        var depth = 0
        while let byte = current {
            switch byte {
            case UInt8(ascii: "["), UInt8(ascii: "{"): depth += 1
            case UInt8(ascii: "]"), UInt8(ascii: "}"):
                if depth == 0 { return true }
                depth -= 1
            case UInt8(ascii: ","):
                if depth == 0 { return true }
            case UInt8(ascii: "\""):
                guard string() != nil else { return false }
                continue
            case 0x20, 0x0A, 0x0D, 0x09:
                if depth == 0 { return true }
            default: break
            }
            index += 1
        }
        return depth == 0
    }
}
