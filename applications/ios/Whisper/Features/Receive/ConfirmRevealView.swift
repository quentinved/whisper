import SwiftUI

/// The deliberate pause before a one-time secret is consumed.
///
/// On a phone this matters more than on the web: link previews, prefetching and
/// share-sheet handoff can all open a URL that the user only meant to look at.
/// Nothing here touches the consuming endpoint until the button is pressed.
struct ConfirmRevealView: View {
    let link: WhisperLink?
    let metadata: SecretMetadata?
    let isWorking: Bool
    let onReveal: () -> Void
    let onCancel: () -> Void

    /// `nil` when the server is too old to tell us, in which case the copy
    /// hedges rather than promising the secret will survive.
    private var burnsOnRead: Bool? { metadata?.selfDestruct }

    var body: some View {
        VStack(spacing: Theme.Spacing.loose) {
            Image(systemName: burnsOnRead == false ? "envelope" : "flame.fill")
                .font(.system(size: 26, weight: .medium))
                .foregroundStyle(burnsOnRead == false ? Theme.accent : Theme.warning)
                .frame(width: 64, height: 64)
                .background(
                    (burnsOnRead == false ? Theme.accent : Theme.warning).opacity(0.08)
                )
                .clipShape(Circle())
                .overlay(
                    Circle().stroke(
                        (burnsOnRead == false ? Theme.border : Theme.warning.opacity(0.3)),
                        lineWidth: 1
                    )
                )
                .padding(.top, Theme.Spacing.loose)

            VStack(spacing: Theme.Spacing.tight) {
                Text(title)
                    .font(.system(size: 24, weight: .bold))
                    .tracking(-0.4)
                    .foregroundStyle(Theme.textPrimary)
                    .multilineTextAlignment(.center)

                Text(subtitle)
                    .font(.system(size: 15))
                    .foregroundStyle(Theme.textMuted)
                    .multilineTextAlignment(.center)
            }

            if burnsOnRead != false {
                Banner(
                    kind: .warning,
                    message: burnsOnRead == true
                        ? "Once you open it, it is gone from the server for good. Copy it somewhere safe before you leave this screen."
                        : "This secret may be destroyed when it is opened. Copy it somewhere safe before you leave this screen."
                )
            }

            VStack(spacing: Theme.Spacing.snug) {
                Button(action: onReveal) {
                    if isWorking {
                        HStack(spacing: Theme.Spacing.tight) {
                            ProgressView().tint(Theme.textInverse)
                            Text("Decrypting…")
                        }
                    } else {
                        Label(
                            "Reveal secret",
                            systemImage: BiometricGate.isAvailable ? "faceid" : "eye"
                        )
                    }
                }
                .buttonStyle(PrimaryButtonStyle())
                .disabled(isWorking)
                .accessibilityIdentifier("reveal-button")

                Button("Cancel", action: onCancel)
                    .buttonStyle(SecondaryButtonStyle())
                    .disabled(isWorking)
            }
        }
    }

    private var title: String {
        switch burnsOnRead {
        case true: return "Open once, then it's gone"
        case false: return "Ready to open"
        case nil: return "Ready to open"
        }
    }

    private var subtitle: String {
        guard let link else { return "Fetching and decrypting on this device…" }
        return "From \(link.server.host ?? link.server.absoluteString)"
    }
}
