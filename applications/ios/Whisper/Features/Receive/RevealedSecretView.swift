import SwiftUI

/// The secret itself. If it was a one-time secret, what is on screen is now the
/// only copy in existence — the UI says so plainly rather than letting the user
/// discover it by navigating away.
struct RevealedSecretView: View {
    let secret: String
    let burned: Bool
    let onDone: () -> Void

    @State private var didCopy = false
    @State private var isHidden = false

    var body: some View {
        VStack(spacing: Theme.Spacing.loose) {
            if burned {
                Banner(
                    kind: .warning,
                    message: "This secret has been destroyed on the server. What you see below is the only copy left."
                )
            }

            VStack(alignment: .leading, spacing: Theme.Spacing.snug) {
                HStack {
                    FieldLabel(text: "Secret")
                    Spacer()
                    Button(isHidden ? "Show" : "Hide") {
                        withAnimation(.easeOut(duration: 0.2)) { isHidden.toggle() }
                    }
                    .font(.system(size: 13, weight: .medium))
                    .foregroundStyle(Theme.textMuted)
                }

                Text(secret)
                    .font(.system(size: 16, design: .monospaced))
                    .foregroundStyle(Theme.textPrimary)
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .accessibilityIdentifier("revealed-secret")
                    .blur(radius: isHidden ? 9 : 0)
                    .animation(.easeOut(duration: 0.2), value: isHidden)
            }
            .cardBackground(padding: Theme.Spacing.normal)

            VStack(spacing: Theme.Spacing.snug) {
                Button {
                    Clipboard.copyExpiring(secret)
                    UINotificationFeedbackGenerator().notificationOccurred(.success)
                    withAnimation(.spring(response: 0.3)) { didCopy = true }
                } label: {
                    Label(
                        didCopy ? "Copied" : "Copy secret",
                        systemImage: didCopy ? "checkmark" : "doc.on.doc"
                    )
                    .contentTransition(.symbolEffect(.replace))
                }
                .buttonStyle(PrimaryButtonStyle())

                Button("Done", action: onDone)
                    .buttonStyle(SecondaryButtonStyle())
            }

            Text("Copies clear from the clipboard after \(Int(Clipboard.lifetime)) seconds.")
                .font(.system(size: 13))
                .foregroundStyle(Theme.textMuted)
                .frame(maxWidth: .infinity)
                .multilineTextAlignment(.center)
        }
        .padding(.top, Theme.Spacing.tight)
    }
}
