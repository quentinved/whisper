import SwiftUI
import WhisperCore

/// A short swipeable tour shown on first launch. Each page previews the piece
/// of the app it talks about, built from the same components. Replayable from
/// Settings.
struct OnboardingView: View {
    /// `@AppStorage` key; `true` until the tour is finished or skipped.
    static let pendingKey = "onboarding_pending"

    private enum Scene {
        case hero, compose, privacy, track
    }

    private struct Page {
        let scene: Scene
        let title: String
        let text: String
    }

    private static let pages = [
        Page(
            scene: .hero,
            title: "Say it once.\nThen it's gone.",
            text: "Send a password or private note as a link that deletes itself once it's read."),
        Page(
            scene: .compose,
            title: "Ready in seconds",
            text: "Type your secret, choose how long it lasts and share the link. No sign-up."),
        Page(
            scene: .privacy,
            title: "Only your link opens it",
            text: "Encrypted on your iPhone. The key lives in the link, never on our server."),
        Page(
            scene: .track,
            title: "Know when it's read",
            text: "The Sent tab shows when each link is opened. Got one? Just tap it."),
    ]

    let onFinish: () -> Void
    @State private var index = 0

    private var isLast: Bool { index == Self.pages.count - 1 }

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Spacer()
                Button("Skip", action: onFinish)
                    .font(.system(size: 16, weight: .medium))
                    .foregroundStyle(Theme.textSecondary)
                    .opacity(isLast ? 0 : 1)
                    .disabled(isLast)
                    .accessibilityHidden(isLast)
                    .accessibilityIdentifier("onboarding-skip")
            }
            .padding(.horizontal, Theme.Spacing.loose)
            .padding(.top, Theme.Spacing.tight)

            TabView(selection: $index) {
                ForEach(Self.pages.indices, id: \.self) { i in
                    page(Self.pages[i]).tag(i)
                }
            }
            .tabViewStyle(.page(indexDisplayMode: .never))

            VStack(spacing: Theme.Spacing.loose) {
                PageDots(count: Self.pages.count, index: index)

                Button {
                    if isLast {
                        onFinish()
                    } else {
                        withAnimation { index += 1 }
                    }
                } label: {
                    HStack(spacing: Theme.Spacing.tight) {
                        Text(isLast ? "Get started" : "Continue")
                        Image(systemName: "arrow.right")
                            .font(.system(size: 15, weight: .bold))
                            .accessibilityHidden(true)
                    }
                }
                .buttonStyle(PrimaryButtonStyle())
                .accessibilityIdentifier("onboarding-next")
            }
            .padding(.horizontal, Theme.Spacing.loose)
            .padding(.bottom, Theme.Spacing.normal)
        }
        .background(AuroraBackground())
    }

    private func page(_ page: Page) -> some View {
        VStack(spacing: Theme.Spacing.section) {
            scene(page.scene)
                .modifier(Floating())
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .background(
                    Circle()
                        .fill(Theme.violet)
                        .frame(width: 220, height: 220)
                        .blur(radius: 70)
                        .opacity(0.22)
                )
                .accessibilityHidden(true)

            VStack(spacing: Theme.Spacing.snug) {
                Text(page.title)
                    .font(.system(size: 30, weight: .bold))
                    .tracking(-0.6)
                    .foregroundStyle(Theme.textPrimary)
                Text(page.text)
                    .font(.system(size: 17))
                    .foregroundStyle(Theme.textSecondary)
                    .fixedSize(horizontal: false, vertical: true)
            }
            .multilineTextAlignment(.center)
            .accessibilityElement(children: .combine)
        }
        .padding(.horizontal, Theme.Spacing.loose)
        .padding(.bottom, Theme.Spacing.loose)
    }

    @ViewBuilder
    private func scene(_ scene: Scene) -> some View {
        switch scene {
        case .hero: heroScene
        case .compose: composeScene
        case .privacy: privacyScene
        case .track: trackScene
        }
    }

    // MARK: Scenes — miniatures of the real screens

    private var heroScene: some View {
        ZStack {
            GlassBadge(size: 128, cornerRadius: 38) {
                BrandMark(size: 68)
            }
            Chip(text: "Password", systemImage: "key.fill", tint: Theme.violet)
                .offset(x: -86, y: -104)
            Chip(text: "API key", systemImage: "curlybraces", tint: Theme.indigo)
                .offset(x: 104, y: -30)
            Chip(text: "Private note", systemImage: "note.text", tint: Theme.mint)
                .offset(x: -64, y: 104)
        }
    }

    private var composeScene: some View {
        VStack(alignment: .leading, spacing: Theme.Spacing.snug) {
            FieldLabel(text: "Your secret")
            Text("••••••••••••••")
                .font(.system(size: 17, weight: .semibold, design: .monospaced))
                .foregroundStyle(Theme.textPrimary)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(Theme.Spacing.snug)
                .background(
                    Theme.input.opacity(0.6),
                    in: RoundedRectangle(cornerRadius: Theme.Radius.small, style: .continuous))
            HStack(spacing: Theme.Spacing.tight) {
                Chip(text: "1 hour", systemImage: "clock")
                Chip(text: "Opens once", systemImage: "flame.fill", tint: Theme.warning)
            }
            Label("Create secret link", systemImage: "link")
                .font(.system(size: 15, weight: .semibold))
                .foregroundStyle(Theme.onBrand)
                .frame(maxWidth: .infinity)
                .frame(height: 44)
                .background(
                    Theme.brandGradient,
                    in: RoundedRectangle(cornerRadius: Theme.Radius.medium, style: .continuous))
        }
        .cardBackground()
        .frame(maxWidth: 300)
    }

    private var privacyScene: some View {
        VStack(spacing: Theme.Spacing.loose) {
            GlassBadge(size: 104, cornerRadius: 32) {
                Image(systemName: "lock.fill")
                    .font(.system(size: 42, weight: .medium))
                    .foregroundStyle(Theme.markGradient)
            }
            HStack(spacing: Theme.Spacing.tight) {
                Image(systemName: "link")
                    .foregroundStyle(Theme.textMuted)
                Text("whisper…/secret")
                    .foregroundStyle(Theme.textSecondary)
                Label("#key", systemImage: "key.fill")
                    .font(.system(size: 14, weight: .semibold, design: .monospaced))
                    .foregroundStyle(Theme.mint)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 5)
                    .background(Theme.mint.opacity(0.14), in: Capsule())
            }
            .font(.system(size: 14, design: .monospaced))
            .padding(.leading, Theme.Spacing.normal)
            .padding(.trailing, 6)
            .frame(height: 44)
            .glassPanel(cornerRadius: Theme.Radius.pill)
        }
    }

    private var trackScene: some View {
        VStack(alignment: .leading, spacing: Theme.Spacing.snug) {
            sentRow(
                "Staging database",
                status: DeliveryStatus(
                    state: .opened, expiresAt: .now.addingTimeInterval(3600),
                    openedAt: .now.addingTimeInterval(-120)))
            Divider()
            sentRow(
                "Wi-Fi password",
                status: DeliveryStatus(
                    state: .waiting, expiresAt: .now.addingTimeInterval(86_400), openedAt: nil))
        }
        .cardBackground()
        .frame(maxWidth: 300)
    }

    private func sentRow(_ name: String, status: DeliveryStatus) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(name)
                .font(.system(size: 16, weight: .semibold))
                .foregroundStyle(Theme.textPrimary)
            DeliveryStatusLabel(status: .known(status), selfDestruct: true)
        }
    }
}

/// The tour's progress: the current page stretches into a brand-coloured pill.
private struct PageDots: View {
    let count: Int
    let index: Int

    var body: some View {
        HStack(spacing: 6) {
            ForEach(0..<count, id: \.self) { i in
                Capsule()
                    .fill(
                        i == index
                            ? AnyShapeStyle(Theme.brandGradient)
                            : AnyShapeStyle(Theme.textMuted.opacity(0.35))
                    )
                    .frame(width: i == index ? 24 : 7, height: 7)
            }
        }
        .animation(.spring(response: 0.35, dampingFraction: 0.8), value: index)
        .accessibilityElement()
        .accessibilityLabel("Page \(index + 1) of \(count)")
    }
}

/// A slow bob so the previews feel alive. Still when Reduce Motion is on.
private struct Floating: ViewModifier {
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @State private var up = false

    func body(content: Content) -> some View {
        content
            .offset(y: up ? -6 : 6)
            .onAppear {
                guard !reduceMotion else { return }
                withAnimation(.easeInOut(duration: 3).repeatForever(autoreverses: true)) {
                    up = true
                }
            }
    }
}
