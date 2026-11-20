import SwiftUI

/// The app icon's light, behind every screen: soft violet, indigo and mint
/// glows drifting slowly over midnight (or porcelain in light mode). Still when
/// Reduce Motion is on.
struct AuroraBackground: View {
    @Environment(\.colorScheme) private var scheme
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @State private var drift = false

    var body: some View {
        ZStack {
            Theme.backgroundGradient
            glow(Theme.violet, size: 440)
                .offset(x: drift ? -70 : -150, y: drift ? -300 : -230)
            glow(Theme.indigo, size: 380)
                .offset(x: drift ? 170 : 110, y: drift ? 20 : 110)
            glow(Theme.mint, size: 320)
                .offset(x: drift ? -160 : -80, y: drift ? 360 : 420)
        }
        .ignoresSafeArea()
        .allowsHitTesting(false)
        .onAppear {
            guard !reduceMotion else { return }
            withAnimation(.easeInOut(duration: 12).repeatForever(autoreverses: true)) {
                drift = true
            }
        }
    }

    private func glow(_ color: Color, size: CGFloat) -> some View {
        Circle()
            .fill(color)
            .frame(width: size, height: size)
            .blur(radius: 110)
            .opacity(scheme == .dark ? 0.30 : 0.14)
    }
}
