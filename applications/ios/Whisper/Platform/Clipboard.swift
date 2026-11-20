import UIKit
import UniformTypeIdentifiers

enum Clipboard {
    /// How long a copied secret stays on the pasteboard. iOS clears it for us
    /// at the expiry date, which also keeps it out of Universal Clipboard on
    /// the user's other devices after that point.
    static let lifetime: TimeInterval = 90

    /// Copies text that should not outlive its usefulness. Used for revealed
    /// secrets — never for share links, which the user may need for longer.
    static func copyExpiring(_ text: String) {
        UIPasteboard.general.setItems(
            [[UTType.utf8PlainText.identifier: text]],
            options: [.expirationDate: Date().addingTimeInterval(lifetime)]
        )
    }

    static func copy(_ text: String) {
        UIPasteboard.general.string = text
    }

    /// The pasteboard's contents, if they look like they could be a share link.
    /// Read without `UIPasteboard.general.string` where possible so iOS doesn't
    /// show the "pasted from" banner until the user actually asks to paste.
    static var probableLink: String? {
        guard UIPasteboard.general.hasStrings else { return nil }
        return UIPasteboard.general.string
    }
}
