import SwiftUI
import WhisperCore

/// The secret itself — frosted until the user chooses to look, so opening a
/// link on a train doesn't show it to the person next to you. It can be copied
/// without ever being shown.
struct RevealedSecretView: View {
    let secret: String
    let burned: Bool
    let onDone: () -> Void

    @State private var didCopy = false
    @State private var isHidden = true

    /// Set when the secret is a JSON object, as the web's Multiple Values sends.
    private var structured: StructuredSecret? { StructuredSecret(parsing: secret) }

    var body: some View {
        VStack(spacing: Theme.Spacing.loose) {
            VStack(spacing: Theme.Spacing.snug) {
                Text("Here's your secret")
                    .font(.system(size: 28, weight: .bold))
                    .tracking(-0.5)
                    .foregroundStyle(Theme.textPrimary)

                if burned {
                    Chip(
                        text: "Deleted from the server — this is the only copy",
                        systemImage: "flame.fill", tint: Theme.warning)
                }
            }
            .padding(.top, Theme.Spacing.loose)

            if let structured {
                StructuredSecretCard(secret: structured)
            } else {
                secretCard
            }

            VStack(spacing: Theme.Spacing.snug) {
                Button {
                    Clipboard.copyExpiring(secret)
                    UINotificationFeedbackGenerator().notificationOccurred(.success)
                    withAnimation(.spring(response: 0.3)) { didCopy = true }
                } label: {
                    Label(
                        didCopy ? "Copied" : (structured == nil ? "Copy secret" : "Copy all as JSON"),
                        systemImage: didCopy ? "checkmark" : "doc.on.doc"
                    )
                    .contentTransition(.symbolEffect(.replace))
                }
                .buttonStyle(PrimaryButtonStyle())

                Button("Done", action: onDone)
                    .buttonStyle(SecondaryButtonStyle())
            }

            Text("The clipboard clears itself after \(Int(Clipboard.lifetime)) seconds.")
                .font(.system(size: 13))
                .foregroundStyle(Theme.textMuted)
        }
    }

    private var secretCard: some View {
        VStack(alignment: .leading, spacing: Theme.Spacing.snug) {
            HStack {
                FieldLabel(text: "Secret")
                Spacer()
                Button {
                    toggle()
                } label: {
                    Label(isHidden ? "Show" : "Hide", systemImage: isHidden ? "eye" : "eye.slash")
                        .font(.system(size: 13, weight: .medium))
                        .foregroundStyle(Theme.accent)
                }
                .accessibilityIdentifier("reveal-toggle")
            }

            ZStack {
                Text(secret)
                    .font(.system(size: 16, design: .monospaced))
                    .foregroundStyle(Theme.textPrimary)
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, minHeight: 72, alignment: .leading)
                    .blur(radius: isHidden ? 14 : 0)
                    // While frosted, VoiceOver must not read it aloud either.
                    .accessibilityHidden(isHidden)
                    .accessibilityIdentifier("revealed-secret")

                if isHidden {
                    Button(action: toggle) {
                        VStack(spacing: 6) {
                            Image(systemName: "eye.fill")
                                .font(.system(size: 22))
                                .foregroundStyle(Theme.markGradient)
                            Text("Tap to reveal")
                                .font(.system(size: 14, weight: .semibold))
                                .foregroundStyle(Theme.textPrimary)
                        }
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                        .contentShape(Rectangle())
                    }
                    .buttonStyle(.plain)
                    .transition(.opacity)
                }
            }
        }
        .cardBackground()
    }

    private func toggle() {
        UIImpactFeedbackGenerator(style: .soft).impactOccurred()
        withAnimation(.easeOut(duration: 0.35)) { isHidden.toggle() }
    }
}
