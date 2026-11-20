import CoreImage.CIFilterBuiltins
import SwiftUI
import WhisperCore

/// Shown once a link exists. The link is the only copy of the key, so this
/// screen's job is to get it somewhere safe — without throwing a wall of
/// base64 at the user. The URL is there, one line, for anyone who wants it.
struct LinkResultView: View {
    let link: WhisperLink
    let expiration: SecretExpiration
    let selfDestruct: Bool
    /// Present when the server tracks delivery for this link.
    let sentSecret: SentSecret?
    let onCreateAnother: () -> Void

    @Environment(SentStore.self) private var sent
    @Environment(\.scenePhase) private var scenePhase
    @State private var didCopy = false
    @State private var showingQR = false
    @State private var sealScale: CGFloat = 0.6

    var body: some View {
        VStack(spacing: Theme.Spacing.loose) {
            seal

            VStack(spacing: Theme.Spacing.snug) {
                Text("Your link is ready")
                    .font(.system(size: 28, weight: .bold))
                    .tracking(-0.5)
                    .foregroundStyle(Theme.textPrimary)

                HStack(spacing: Theme.Spacing.tight) {
                    Chip(text: expiration.label, systemImage: "timer")
                    if selfDestruct {
                        Chip(text: "Opens once", systemImage: "flame.fill", tint: Theme.warning)
                    } else {
                        Chip(text: "Reusable", systemImage: "arrow.triangle.2.circlepath")
                    }
                }
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

                HStack(spacing: Theme.Spacing.snug) {
                    ShareLink(item: link.url) {
                        Label("Share", systemImage: "square.and.arrow.up")
                    }
                    .buttonStyle(SecondaryButtonStyle())

                    Button {
                        showingQR = true
                    } label: {
                        Label("QR code", systemImage: "qrcode")
                    }
                    .buttonStyle(SecondaryButtonStyle())
                }
            }

            Button("Create another secret", action: onCreateAnother)
                .font(.system(size: 15, weight: .medium))
                .foregroundStyle(Theme.textMuted)
        }
        .sheet(isPresented: $showingQR) {
            QRCodeSheet(url: link.url)
                .presentationDetents([.medium])
                .presentationBackground(.ultraThinMaterial)
        }
        .onAppear {
            withAnimation(.spring(response: 0.5, dampingFraction: 0.6)) { sealScale = 1 }
        }
    }

    private var seal: some View {
        Image(systemName: "checkmark")
            .font(.system(size: 32, weight: .bold))
            .foregroundStyle(Theme.onBrand)
            .frame(width: 80, height: 80)
            .background(Theme.brandGradient, in: Circle())
            .shadow(color: Theme.violet.opacity(0.3), radius: 14)
            .scaleEffect(sealScale)
            .padding(.top, Theme.Spacing.normal)
    }

    private var linkCard: some View {
        HStack(spacing: Theme.Spacing.snug) {
            Image(systemName: "lock.fill")
                .font(.system(size: 18, weight: .semibold))
                .foregroundStyle(Theme.onBrand)
                .frame(width: 44, height: 44)
                .background(
                    Theme.brandGradient,
                    in: RoundedRectangle(cornerRadius: Theme.Radius.medium, style: .continuous))

            VStack(alignment: .leading, spacing: 3) {
                Text("Secret link")
                    .font(.system(size: 16, weight: .semibold))
                    .foregroundStyle(Theme.textPrimary)
                // The full URL, kept to one line: the key is in there, but no
                // one needs to read it.
                Text(link.url.absoluteString)
                    .font(.system(size: 12, design: .monospaced))
                    .foregroundStyle(Theme.textMuted)
                    .lineLimit(1)
                    .truncationMode(.middle)
                    .textSelection(.enabled)
                    .accessibilityIdentifier("share-link")
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .cardBackground(padding: Theme.Spacing.snug)
    }
}

extension LinkResultView {
    /// Refreshed on appear and whenever the app comes back to the foreground —
    /// typically right after the sender pasted the link somewhere.
    @ViewBuilder
    fileprivate var deliveryCard: some View {
        if let entry = sentSecret {
            VStack(alignment: .leading, spacing: 4) {
                Text("Delivery")
                    .font(.system(size: 13, weight: .medium))
                    .foregroundStyle(Theme.textMuted)
                DeliveryStatusLabel(status: sent.statuses[entry.token])
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            .cardBackground(padding: Theme.Spacing.snug)
            .task { await sent.refresh(entry) }
            .onChange(of: scenePhase) { _, phase in
                if phase == .active { Task { await sent.refresh(entry) } }
            }
        }
    }
}

/// The link as a QR code, for handing a secret to someone in the room.
private struct QRCodeSheet: View {
    let url: URL

    var body: some View {
        VStack(spacing: Theme.Spacing.normal) {
            Text("Scan to open")
                .font(.system(size: 20, weight: .bold))
                .foregroundStyle(Theme.textPrimary)

            if let image = Self.qrCode(for: url.absoluteString) {
                Image(uiImage: image)
                    .interpolation(.none)
                    .resizable()
                    .scaledToFit()
                    .frame(width: 220, height: 220)
                    .padding(Theme.Spacing.normal)
                    .background(.white, in: RoundedRectangle(cornerRadius: Theme.Radius.large))
                    .accessibilityLabel("QR code of the secret link")
            }

            Text("Only scan it where you trust who's looking.")
                .font(.system(size: 13))
                .foregroundStyle(Theme.textMuted)
        }
        .padding(Theme.Spacing.loose)
    }

    static func qrCode(for text: String) -> UIImage? {
        let filter = CIFilter.qrCodeGenerator()
        filter.message = Data(text.utf8)
        filter.correctionLevel = "M"
        guard let output = filter.outputImage,
            let cgImage = CIContext().createCGImage(output, from: output.extent)
        else { return nil }
        return UIImage(cgImage: cgImage)
    }
}
