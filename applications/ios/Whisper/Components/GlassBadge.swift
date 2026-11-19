import SwiftUI

/// A small floating tile in the system's Liquid Glass material — the treatment
/// iOS 26 uses for chrome that sits above content, as in Safari's toolbar.
///
/// The effect only exists on iOS 26, and the app deploys back to 17, so older
/// releases fall back to a thin material with a hairline border. That fallback
/// is intentionally close rather than identical: on iOS 17–18 there is no
/// specular edge to imitate, and faking one looks worse than not having it.
struct GlassBadge<Content: View>: View {
    var size: CGFloat = 64
    var cornerRadius: CGFloat = 20
    @ViewBuilder var content: Content

    var body: some View {
        content
            .frame(width: size, height: size)
            .modifier(GlassSurface(cornerRadius: cornerRadius))
    }
}

private struct GlassSurface: ViewModifier {
    let cornerRadius: CGFloat

    func body(content: Content) -> some View {
        let shape = RoundedRectangle(cornerRadius: cornerRadius, style: .continuous)

        if #available(iOS 26.0, *) {
            content.glassEffect(.regular, in: shape)
        } else {
            content
                .background(.ultraThinMaterial, in: shape)
                .overlay(shape.stroke(Theme.border, lineWidth: 1))
        }
    }
}
