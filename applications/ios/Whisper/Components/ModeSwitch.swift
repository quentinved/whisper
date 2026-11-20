import SwiftUI

/// Two or three exclusive options in a glass capsule; the chosen one slides a
/// brand-gradient pill under itself. The web's `.mode-switch`, with more glow.
struct ModeSwitch<Option: Hashable>: View {
    let options: [Option]
    @Binding var selection: Option
    let title: (Option) -> String

    @Namespace private var pill

    var body: some View {
        HStack(spacing: 4) {
            ForEach(options, id: \.self) { option in
                let isSelected = option == selection
                Button {
                    UISelectionFeedbackGenerator().selectionChanged()
                    withAnimation(.spring(response: 0.3, dampingFraction: 0.8)) { selection = option }
                } label: {
                    Text(title(option))
                        .font(.system(size: 14, weight: .semibold))
                        .foregroundStyle(isSelected ? Theme.onBrand : Theme.textSecondary)
                        .frame(maxWidth: .infinity)
                        .frame(height: 34)
                        .background {
                            if isSelected {
                                Capsule()
                                    .fill(Theme.brandGradient)
                                    .shadow(color: Theme.violet.opacity(0.4), radius: 8)
                                    .matchedGeometryEffect(id: "pill", in: pill)
                            }
                        }
                        .contentShape(Capsule())
                }
                .buttonStyle(.plain)
                .accessibilityAddTraits(isSelected ? .isSelected : [])
            }
        }
        .padding(4)
        .glassPanel(cornerRadius: 21)
    }
}
