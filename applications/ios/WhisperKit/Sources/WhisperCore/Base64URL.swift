import Foundation

/// base64url without padding (RFC 4648 §5) — the encoding Whisper uses on the
/// wire for ephemeral payloads and for the `#k=` key carried in a share link.
///
/// Foundation only ships standard base64, so the two alphabets are translated
/// by hand here. This must stay byte-compatible with the Rust `base64-url`
/// crate and the browser's `base64url` helpers.
public enum Base64URL {
    public static func encode(_ data: Data) -> String {
        data.base64EncodedString()
            .replacingOccurrences(of: "+", with: "-")
            .replacingOccurrences(of: "/", with: "_")
            .replacingOccurrences(of: "=", with: "")
    }

    public static func decode(_ string: String) -> Data? {
        var standard =
            string
            .replacingOccurrences(of: "-", with: "+")
            .replacingOccurrences(of: "_", with: "/")

        // Restore the padding base64url drops. A remainder of 1 can never be
        // produced by a valid encoder, so reject it rather than guessing.
        switch standard.count % 4 {
        case 0: break
        case 2: standard += "=="
        case 3: standard += "="
        default: return nil
        }

        return Data(base64Encoded: standard)
    }
}
