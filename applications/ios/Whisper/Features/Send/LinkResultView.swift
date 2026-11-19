import SwiftUI

/// Shown once a link exists. The link is the only copy of the key, so this
/// screen's job is to get it somewhere safe before the user leaves.
struct LinkResultView: View {
    let link: WhisperLink
    let expiration: Expiration
    let selfDestruct: Bool
    /// Present when the server tracks delivery for this link.
    let sentSecret: SentSecret?
    let onCreateAnother: () -> Void

    @Environment(SentStore.self) private var sent
    @Environment(\.scenePhase) private var scenePhase
    @State private var didCopy = false
    @State private var sealScale: CGFloat = 0.6

    var body: some View {
        VStack(spacing: Theme.Spacing.loose) {
            seal

            VStack(spacing: Theme.Spacing.tight) {
                Text("Link ready")
                    .font(.system(size: 26, weight: .bold))
                    .tracking(-0.4)
                    .foregroundStyle(Theme.textPrimary)

                Text(summary)
                    .font(.system(size: 15))
                    .foregroundStyle(Theme.textMuted)
                    .multilineTextAlignment(.center)
            }

            linkCard

            deliveryCard

            VStack(spacing: Theme.Spacing.snug) {
                Button {
                    Clipboard.copy(link.url.absoluteString)
                    UINotificationFeedbackGenerator().notificationOccurred(.success)
                    withAnimation(.spring(response: 0.3)) { didCopy = true }
                } label: {
                    Label(
                        didCopy ? "Copied" : "Copy link",
                        systemImage: didCopy ? "checkmark" : "doc.on.doc"
                    )
                    .contentTransition(.symbolEffect(.replace))
                }
                .buttonStyle(PrimaryButtonStyle())

                ShareLink(item: link.url) {
                    Label("Share", systemImage: "square.and.arrow.up")
                }
                .buttonStyle(SecondaryButtonStyle())
            }

            Button("Create another secret", action: onCreateAnother)
                .font(.system(size: 15, weight: .medium))
                .foregroundStyle(Theme.textMuted)

            keyNotice
        }
        .onAppear {
            withAnimation(.spring(response: 0.5, dampingFraction: 0.6)) { sealScale = 1 }
        }
    }

    private var summary: String {
        let lifetime = "Expires in \(expiration.label.lowercased())"
        return selfDestruct
            ? "\(lifetime), or the moment someone opens it."
            : "\(lifetime). It can be opened more than once."
    }

    private var seal: some View {
        Image(systemName: "checkmark")
            .font(.system(size: 30, weight: .semibold))
            .foregroundStyle(Theme.textInverse)
            .frame(width: 72, height: 72)
            .background(Theme.accent)
            .clipShape(Circle())
            .scaleEffect(sealScale)
            .padding(.top, Theme.Spacing.normal)
    }

    private var linkCard: some View {
        VStack(alignment: .leading, spacing: Theme.Spacing.tight) {
            FieldLabel(text: "Share link")
            Text(link.url.absoluteString)
                .font(.system(size: 13, design: .monospaced))
                .foregroundStyle(Theme.textSecondary)
                .textSelection(.enabled)
                .lineLimit(4)
                .truncationMode(.middle)
                .accessibilityIdentifier("share-link")
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .cardBackground()
    }

    /// Refreshed on appear and whenever the app comes back to the foreground —
    /// typically right after the sender pasted the link somewhere.
    @ViewBuilder
    private var deliveryCard: some View {
        if let entry = sentSecret {
            VStack(alignment: .leading, spacing: Theme.Spacing.tight) {
                FieldLabel(text: "Delivery")
                DeliveryStatusLabel(status: sent.statuses[entry.token])
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .cardBackground()
            .task { await sent.refresh(entry) }
            .onChange(of: scenePhase) { _, phase in
                if phase == .active { Task { await sent.refresh(entry) } }
            }
        }
    }

    private var keyNotice: some View {
        HStack(alignment: .top, spacing: Theme.Spacing.snug) {
            Image(systemName: "key.fill")
                .font(.system(size: 13))
                .foregroundStyle(Theme.textMuted)
            Text("The decryption key is the part after `#`. It stays in the link — it was never sent to the server, so only whoever holds this link can read the secret.")
                .font(.system(size: 13))
                .foregroundStyle(Theme.textMuted)
        }
        .padding(.horizontal, Theme.Spacing.tight)
    }
}
