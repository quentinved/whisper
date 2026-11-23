import SwiftUI

/// A short swipeable tour shown on first launch: what Whisper is, then one page
/// per tab. Replayable from Settings.
struct OnboardingView: View {
    /// `@AppStorage` key; `true` until the tour is finished or skipped.
    static let pendingKey = "onboarding_pending"

    private struct Page {
        /// `nil` draws the Whisper mark instead of an SF Symbol.
        let icon: String?
        let title: String
        let text: String
    }

    private static let pages = [
        Page(
            icon: nil,
            title: "Share secrets that disappear",
            text: "Send passwords, API keys and private notes as a link that deletes itself. No account needed."),
        Page(
            icon: "paperplane.fill",
            title: "Send",
            text: "Write your secret, choose when it expires and keep Burn after reading on so it opens only once. Then tap Create secret link and share it."),
        Page(
            icon: "lock.fill",
            title: "Only the link can open it",
            text: "Your secret is encrypted on this iPhone. The key is inside the link and never reaches the server, so send the link only to the person who needs it."),
        Page(
            icon: "tray.full.fill",
            title: "Sent",
            text: "See whether each secret you sent has been opened yet."),
        Page(
            icon: "envelope.open.fill",
            title: "Open",
            text: "Tap a Whisper link, or paste it in the Open tab. You confirm before it's shown — a one-time secret is deleted as soon as you see it."),
    ]

    let onFinish: () -> Void
    @State private var index = 0

    private var isLast: Bool { index == Self.pages.count - 1 }

    var body: some View {
        VStack(spacing: Theme.Spacing.loose) {
            HStack {
                Spacer()
                Button("Skip", action: onFinish)
                    .foregroundStyle(Theme.textSecondary)
                    .opacity(isLast ? 0 : 1)
                    .disabled(isLast)
                    .accessibilityHidden(isLast)
                    .accessibilityIdentifier("onboarding-skip")
            }

            TabView(selection: $index) {
                ForEach(Self.pages.indices, id: \.self) { i in
                    page(Self.pages[i]).tag(i)
                }
            }
            .tabViewStyle(.page(indexDisplayMode: .always))
            // The default white dots vanish on the light-mode backdrop.
            .indexViewStyle(.page(backgroundDisplayMode: .always))

            Button(isLast ? "Get started" : "Next") {
                if isLast {
                    onFinish()
                } else {
                    withAnimation { index += 1 }
                }
            }
            .buttonStyle(PrimaryButtonStyle())
            .accessibilityIdentifier("onboarding-next")
        }
        .padding(Theme.Spacing.normal)
        .background(AuroraBackground())
    }

    private func page(_ page: Page) -> some View {
        VStack(spacing: Theme.Spacing.loose) {
            GlassBadge(size: 96, cornerRadius: 30) {
                if let icon = page.icon {
                    Image(systemName: icon)
                        .font(.system(size: 36, weight: .medium))
                        .foregroundStyle(Theme.markGradient)
                } else {
                    BrandMark(size: 50)
                }
            }
            .accessibilityHidden(true)

            VStack(spacing: Theme.Spacing.snug) {
                Text(page.title)
                    .font(.system(size: 28, weight: .bold))
                    .tracking(-0.5)
                    .foregroundStyle(Theme.textPrimary)
                Text(page.text)
                    .font(.system(size: 17))
                    .foregroundStyle(Theme.textSecondary)
            }
            .multilineTextAlignment(.center)
            .accessibilityElement(children: .combine)
        }
        .padding(.horizontal, Theme.Spacing.normal)
        .padding(.bottom, Theme.Spacing.section)
    }
}
