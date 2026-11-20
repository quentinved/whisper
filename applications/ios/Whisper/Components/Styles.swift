import SwiftUI

/// The filled brand action. There is at most one per screen.
struct PrimaryButtonStyle: ButtonStyle {
    @Environment(\.isEnabled) private var isEnabled

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.system(size: 17, weight: .semibold))
            .foregroundStyle(Theme.onBrand)
            .frame(maxWidth: .infinity)
            .frame(height: 54)
            .background(Theme.brandGradient)
            .clipShape(RoundedRectangle(cornerRadius: Theme.Radius.medium, style: .continuous))
            .shadow(color: Theme.violet.opacity(isEnabled ? 0.25 : 0), radius: 10, y: 4)
            .opacity(isEnabled ? 1 : 0.4)
            .scaleEffect(configuration.isPressed ? 0.97 : 1)
            .animation(.spring(response: 0.3, dampingFraction: 0.7), value: configuration.isPressed)
    }
}

/// Glass counterpart for secondary actions sitting next to a primary one.
struct SecondaryButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.system(size: 17, weight: .medium))
            .foregroundStyle(Theme.textPrimary)
            .frame(maxWidth: .infinity)
            .frame(height: 54)
            .glassPanel(cornerRadius: Theme.Radius.medium)
            .scaleEffect(configuration.isPressed ? 0.97 : 1)
            .animation(.spring(response: 0.3, dampingFraction: 0.7), value: configuration.isPressed)
    }
}

extension View {
    /// Frosted glass over the aurora backdrop, with a hairline edge.
    func glassPanel(cornerRadius: CGFloat = Theme.Radius.large) -> some View {
        let shape = RoundedRectangle(cornerRadius: cornerRadius, style: .continuous)
        return
            self
            .background(.ultraThinMaterial, in: shape)
            .overlay(shape.stroke(Theme.border, lineWidth: 1))
    }

    /// The raised panel the web UI draws as `.card`.
    func cardBackground(padding: CGFloat = Theme.Spacing.normal) -> some View {
        self.padding(padding).glassPanel()
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

/// A compact fact about a secret: "1 hour", "Opens once".
struct Chip: View {
    let text: String
    let systemImage: String
    var tint: Color = Theme.textSecondary

    var body: some View {
        Label(text, systemImage: systemImage)
            .font(.system(size: 13, weight: .medium))
            .foregroundStyle(tint)
            .padding(.horizontal, 10)
            .padding(.vertical, 6)
            .background(.ultraThinMaterial, in: Capsule())
            .overlay(Capsule().stroke(tint.opacity(0.25), lineWidth: 1))
    }
}

/// Inline status message: `.warning` for one-time reads, `.error` for failures.
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

/// A scrolling screen over the aurora backdrop. Dragging or the keyboard's
/// Done button puts the keyboard away (taps outside are handled app-wide by
/// `KeyboardDismissal`) — a multi-line editor's Return key only adds a line.
struct Screen<Content: View>: View {
    @ViewBuilder var content: Content

    var body: some View {
        ScrollView {
            content
                .padding(Theme.Spacing.normal)
                .padding(.bottom, Theme.Spacing.section)
        }
        .scrollDismissesKeyboard(.interactively)
        .background(AuroraBackground())
        .toolbar {
            ToolbarItemGroup(placement: .keyboard) {
                Spacer()
                Button("Done", action: dismissKeyboard)
                    .fontWeight(.semibold)
                    .accessibilityIdentifier("keyboard-done")
            }
        }
    }
}

@MainActor
func dismissKeyboard() {
    UIApplication.shared.sendAction(
        #selector(UIResponder.resignFirstResponder), to: nil, from: nil, for: nil)
}
