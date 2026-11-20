import SwiftUI

/// The iOS mirror of the web design system in
/// `applications/axum/server/assets/css/modern.css`, plus one thing the web
/// doesn't have: the brand colours of the app icon (`Tools/sync_brand.swift`,
/// "aurora" style) for actions, toggles and the backdrop.
///
/// Text, borders and status colours map to the CSS custom property of the same
/// name; when one changes there, change it here too.
enum Theme {
    // MARK: Brand — from the app icon

    static let violet = Color.themed(light: 0x7C3AED, dark: 0x8B5CF6)
    static let indigo = Color.themed(light: 0x4F46E5, dark: 0x6366F1)
    static let mint = Color.themed(light: 0x0D9488, dark: 0x5EEAD4)

    /// Primary actions and the brand mark.
    static let brandGradient = LinearGradient(
        colors: [violet, indigo],
        startPoint: .topLeading,
        endPoint: .bottomTrailing
    )

    /// The icon's ribbon: violet into mint.
    static let markGradient = LinearGradient(
        colors: [Color.themed(light: 0x6D28D9, dark: 0xC4B5FD), mint],
        startPoint: .topLeading,
        endPoint: .bottomTrailing
    )

    // MARK: Surfaces

    /// Base of the aurora backdrop: porcelain by day, the icon's midnight by night.
    static let backgroundGradient = LinearGradient(
        colors: [
            .themed(light: 0xFBFAFF, dark: 0x0E0B22),
            .themed(light: 0xF1EFFA, dark: 0x050509),
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
    /// Text and icons drawn on the brand gradient.
    static let onBrand = Color.white

    // MARK: Lines & accent

    /// `--border-color`
    static let border = Color.themed(light: 0xE0E0E0, darkWhiteAlpha: 0.12)
    /// Focused fields glow in the brand colour.
    static let borderFocus = violet
    /// Tint for toggles, tabs and links.
    static let accent = violet

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
