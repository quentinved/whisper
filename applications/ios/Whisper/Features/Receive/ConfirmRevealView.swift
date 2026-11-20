import SwiftUI
import WhisperCore

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
            envelope

            VStack(spacing: Theme.Spacing.snug) {
                Text(isWorking ? "Unsealing…" : "You've got a secret")
                    .font(.system(size: 28, weight: .bold))
                    .tracking(-0.5)
                    .foregroundStyle(Theme.textPrimary)

                if let link {
                    Text("via \(link.server.host ?? link.server.absoluteString)")
                        .font(.system(size: 15))
                        .foregroundStyle(Theme.textMuted)
                }

                if !isWorking {
                    HStack(spacing: Theme.Spacing.tight) {
                        switch burnsOnRead {
                        case true:
                            Chip(text: "Opens once", systemImage: "flame.fill", tint: Theme.warning)
                        case false:
                            Chip(text: "Reusable", systemImage: "arrow.triangle.2.circlepath")
                        case nil:
                            Chip(text: "May open once", systemImage: "flame", tint: Theme.warning)
                        }
                        Chip(text: "Encrypted", systemImage: "lock.fill")
                    }
                }
            }

            if !isWorking, burnsOnRead != false {
                Text(
                    burnsOnRead == true
                        ? "It's deleted the moment you open it."
                        : "It may be deleted once you open it."
                )
                .font(.system(size: 14))
                .foregroundStyle(Theme.textSecondary)
            }

            VStack(spacing: Theme.Spacing.snug) {
                Button(action: onReveal) {
                    if isWorking {
                        HStack(spacing: Theme.Spacing.tight) {
                            ProgressView().tint(Theme.onBrand)
                            Text("Decrypting…")
                        }
                    } else {
                        Label(
                            "Open secret",
                            systemImage: BiometricGate.isAvailable ? "faceid" : "lock.open.fill"
                        )
                    }
                }
                .buttonStyle(PrimaryButtonStyle())
                .disabled(isWorking)
                .accessibilityIdentifier("reveal-button")

                if !isWorking {
                    Button("Not now", action: onCancel)
                        .font(.system(size: 15, weight: .medium))
                        .foregroundStyle(Theme.textMuted)
                        .padding(.top, Theme.Spacing.tight)
                }
            }
        }
    }

    private var envelope: some View {
        GlassBadge(size: 96, cornerRadius: 30) {
            Image(systemName: isWorking ? "envelope.open.fill" : "envelope.fill")
                .font(.system(size: 38, weight: .medium))
                .foregroundStyle(Theme.markGradient)
                .shadow(color: Theme.violet.opacity(0.35), radius: 8)
                .symbolEffect(.pulse, isActive: isWorking)
                .contentTransition(.symbolEffect(.replace))
        }
        .padding(.top, Theme.Spacing.loose)
    }
}
