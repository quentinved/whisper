import SwiftUI

/// The filled, high-contrast action. There is at most one per screen.
struct PrimaryButtonStyle: ButtonStyle {
    @Environment(\.isEnabled) private var isEnabled

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.system(size: 17, weight: .semibold))
            .foregroundStyle(Theme.textInverse)
            .frame(maxWidth: .infinity)
            .frame(height: 54)
            .background(Theme.accent.opacity(isEnabled ? 1 : 0.3))
            .clipShape(RoundedRectangle(cornerRadius: Theme.Radius.medium, style: .continuous))
            .scaleEffect(configuration.isPressed ? 0.97 : 1)
            .animation(.spring(response: 0.3, dampingFraction: 0.7), value: configuration.isPressed)
    }
}

/// Outlined counterpart for secondary actions sitting next to a primary one.
struct SecondaryButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.system(size: 17, weight: .medium))
            .foregroundStyle(Theme.textPrimary)
            .frame(maxWidth: .infinity)
            .frame(height: 54)
            .background(Theme.card)
            .clipShape(RoundedRectangle(cornerRadius: Theme.Radius.medium, style: .continuous))
            .overlay(
                RoundedRectangle(cornerRadius: Theme.Radius.medium, style: .continuous)
                    .stroke(Theme.border, lineWidth: 1)
            )
            .scaleEffect(configuration.isPressed ? 0.97 : 1)
            .animation(.spring(response: 0.3, dampingFraction: 0.7), value: configuration.isPressed)
    }
}

/// The raised panel the web UI draws as `.card`.
struct CardBackground: ViewModifier {
    var padding: CGFloat = Theme.Spacing.normal

    func body(content: Content) -> some View {
        content
            .padding(padding)
            .background(Theme.card)
            .clipShape(RoundedRectangle(cornerRadius: Theme.Radius.large, style: .continuous))
            .overlay(
                RoundedRectangle(cornerRadius: Theme.Radius.large, style: .continuous)
                    .stroke(Theme.border, lineWidth: 1)
            )
    }
}

extension View {
    func cardBackground(padding: CGFloat = Theme.Spacing.normal) -> some View {
        modifier(CardBackground(padding: padding))
    }
}

/// A small uppercase label above a field, matching the web form.
struct FieldLabel: View {
    let text: String

    var body: some View {
        Text(text.uppercased())
            .font(.system(size: 11, weight: .semibold))
            .tracking(0.8)
            .foregroundStyle(Theme.textMuted)
    }
}

/// Inline status message. `.warning` is used for the one-time-read notice and
/// `.error` for failures; nothing else in the app is coloured.
struct Banner: View {
    enum Kind {
        case warning
        case error

        var icon: String {
            switch self {
            case .warning: return "flame.fill"
            case .error: return "exclamationmark.triangle.fill"
            }
        }

        var tint: Color {
            switch self {
            case .warning: return Theme.warning
            case .error: return Theme.error
            }
        }
    }

    @Environment(\.colorScheme) private var scheme
    let kind: Kind
    let message: String

    var body: some View {
        HStack(alignment: .top, spacing: Theme.Spacing.snug) {
            Image(systemName: kind.icon)
                .font(.system(size: 14))
                .foregroundStyle(kind.tint)
            Text(message)
                .font(.system(size: 14))
                .foregroundStyle(Theme.textSecondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(Theme.Spacing.snug)
        .background(
            kind == .warning
                ? Theme.warningBackground(scheme) : Theme.errorBackground(scheme)
        )
        .clipShape(RoundedRectangle(cornerRadius: Theme.Radius.small, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: Theme.Radius.small, style: .continuous)
                .stroke(kind.tint.opacity(0.3), lineWidth: 1)
        )
        .transition(.opacity.combined(with: .move(edge: .top)))
    }
}

extension Banner.Kind: Equatable {}
