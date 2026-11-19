import SwiftUI

/// The iOS mirror of the web design system in
/// `applications/axum/server/assets/css/modern.css`.
///
/// Whisper's look is deliberately monochrome — black on white, white on black,
/// with colour reserved for warnings and errors so that a destructive action is
/// the only thing on screen that is ever coloured. Every value here maps to a
/// CSS custom property of the same name; when one changes there, change it here
/// too rather than inventing a new shade.
enum Theme {
    // MARK: Surfaces

    /// `--bg-gradient-start` → `--bg-gradient-end`
    static let backgroundGradient = LinearGradient(
        colors: [
            .themed(light: 0xFFFFFF, dark: 0x0A0A0A),
            .themed(light: 0xF5F5F5, dark: 0x111111),
        ],
        startPoint: .top,
        endPoint: .bottom
    )

    /// `--bg-card-solid`
    static let card = Color.themed(light: 0xFFFFFF, dark: 0x141414)
    /// `--bg-input`
    static let input = Color.themed(light: 0xFFFFFF, dark: 0x1E1E1E)

    // MARK: Text

    /// `--text-primary`
    static let textPrimary = Color.themed(light: 0x111111, dark: 0xF5F5F5)
    /// `--text-secondary`
    static let textSecondary = Color.themed(light: 0x333333, dark: 0xCCCCCC)
    /// `--text-muted`
    static let textMuted = Color.themed(light: 0x888888, dark: 0x777777)
    /// `--text-inverse` — text drawn on top of `accent`
    static let textInverse = Color.themed(light: 0xFFFFFF, dark: 0x0A0A0A)

    // MARK: Lines & accent

    /// `--border-color`
    static let border = Color.themed(light: 0xE0E0E0, darkWhiteAlpha: 0.12)
    /// `--border-focus`
    static let borderFocus = Color.themed(light: 0x111111, dark: 0xFFFFFF)
    /// `--accent-solid` — the one high-contrast fill, used for primary actions
    static let accent = Color.themed(light: 0x111111, dark: 0xFFFFFF)

    // MARK: Status

    /// `--error`
    static let error = Color.themed(light: 0xDC2626, dark: 0xEF4444)
    /// `--warning` — self-destruct lives here
    static let warning = Color.themed(light: 0xD97706, dark: 0xF59E0B)

    static func errorBackground(_ scheme: ColorScheme) -> Color {
        error.opacity(scheme == .dark ? 0.12 : 0.08)
    }

    static func warningBackground(_ scheme: ColorScheme) -> Color {
        warning.opacity(scheme == .dark ? 0.12 : 0.08)
    }

    // MARK: Geometry — `--radius-*`

    enum Radius {
        static let small: CGFloat = 8
        static let medium: CGFloat = 12
        static let large: CGFloat = 16
        static let pill: CGFloat = 999
    }

    enum Spacing {
        static let tight: CGFloat = 8
        static let snug: CGFloat = 12
        static let normal: CGFloat = 16
        static let loose: CGFloat = 24
        static let section: CGFloat = 32
    }
}

extension Color {
    /// Resolves per-appearance so a single `Color` works in both themes without
    /// every call site having to read `@Environment(\.colorScheme)`.
    static func themed(light: UInt32, dark: UInt32) -> Color {
        Color(
            UIColor { traits in
                UIColor(hex: traits.userInterfaceStyle == .dark ? dark : light)
            })
    }

    /// For the dark-mode tokens that CSS expresses as `rgba(255, 255, 255, a)`.
    static func themed(light: UInt32, darkWhiteAlpha alpha: CGFloat) -> Color {
        Color(
            UIColor { traits in
                traits.userInterfaceStyle == .dark
                    ? UIColor(white: 1, alpha: alpha)
                    : UIColor(hex: light)
            })
    }
}

extension UIColor {
    fileprivate convenience init(hex: UInt32) {
        self.init(
            red: CGFloat((hex >> 16) & 0xFF) / 255,
            green: CGFloat((hex >> 8) & 0xFF) / 255,
            blue: CGFloat(hex & 0xFF) / 255,
            alpha: 1
        )
    }
}
