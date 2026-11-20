import SwiftUI

struct ReceiveView: View {
    @Environment(AppSettings.self) private var settings
    @Bindable var model: ReceiveViewModel
    @FocusState private var fieldFocused: Bool

    var body: some View {
        Screen {
            VStack(spacing: Theme.Spacing.loose) {
                switch model.phase {
                case .idle, .inspecting:
                    inputSection
                case .confirming(let link, let metadata):
                    ConfirmRevealView(
                        link: link,
                        metadata: metadata,
                        isWorking: false,
                        onReveal: { Task { await model.reveal(using: settings) } },
                        onCancel: { withAnimation { model.reset() } }
                    )
                case .revealing:
                    ConfirmRevealView(
                        link: nil,
                        metadata: nil,
                        isWorking: true,
                        onReveal: {},
                        onCancel: {}
                    )
                case .revealed(let secret, let burned):
                    RevealedSecretView(
                        secret: secret,
                        burned: burned,
                        onDone: { withAnimation { model.reset() } }
                    )
                }

                if let message = model.errorMessage {
                    Banner(kind: .error, message: message)
                }
            }
        }
        .animation(.spring(response: 0.4, dampingFraction: 0.85), value: model.phase)
        .animation(.easeOut(duration: 0.2), value: model.errorMessage)
    }

    private var inputSection: some View {
        VStack(alignment: .leading, spacing: Theme.Spacing.loose) {
            VStack(spacing: Theme.Spacing.tight) {
                GlassBadge(size: 76, cornerRadius: 24) {
                    Image(systemName: "envelope.open.fill")
                        .font(.system(size: 28, weight: .medium))
                        .foregroundStyle(Theme.markGradient)
                        .shadow(color: Theme.violet.opacity(0.3), radius: 6)
                }

                Text("Open a secret")
                    .font(.system(size: 28, weight: .bold))
                    .tracking(-0.5)
                    .foregroundStyle(Theme.textPrimary)

                Text("Paste the link someone sent you")
                    .font(.system(size: 15))
                    .foregroundStyle(Theme.textMuted)
            }
            .frame(maxWidth: .infinity)
            .padding(.top, Theme.Spacing.tight)

            VStack(alignment: .leading, spacing: Theme.Spacing.tight) {
                TextField("Paste the link here", text: $model.pasted, axis: .vertical)
                    .font(.system(size: 14, design: .monospaced))
                    .foregroundStyle(Theme.textPrimary)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                    .keyboardType(.URL)
                    .lineLimit(2...5)
                    .focused($fieldFocused)
                    .padding(Theme.Spacing.snug)
                    .background(Theme.input.opacity(0.85))
                    .clipShape(
                        RoundedRectangle(cornerRadius: Theme.Radius.medium, style: .continuous)
                    )
                    .overlay(
                        RoundedRectangle(cornerRadius: Theme.Radius.medium, style: .continuous)
                            .stroke(
                                fieldFocused ? Theme.borderFocus : Theme.border,
                                lineWidth: fieldFocused ? 2 : 1
                            )
                    )
                    .shadow(color: Theme.violet.opacity(fieldFocused ? 0.25 : 0), radius: 12)
                    .animation(.easeOut(duration: 0.15), value: fieldFocused)
                    .accessibilityIdentifier("link-field")
            }

            VStack(spacing: Theme.Spacing.snug) {
                Button {
                    dismissKeyboard()
                    Task { await model.inspect(model.pasted, using: settings) }
                } label: {
                    if model.phase == .inspecting {
                        HStack(spacing: Theme.Spacing.tight) {
                            ProgressView().tint(Theme.onBrand)
                            Text("Checking…")
                        }
                    } else {
                        Text("Continue")
                    }
                }
                .buttonStyle(PrimaryButtonStyle())
                .disabled(model.pasted.isEmpty || model.phase == .inspecting)
                .accessibilityIdentifier("continue-button")

                Button {
                    guard let text = Clipboard.probableLink else { return }
                    model.pasted = text
                    dismissKeyboard()
                    Task { await model.inspect(text, using: settings) }
                } label: {
                    Label("Paste from clipboard", systemImage: "doc.on.clipboard")
                }
                .buttonStyle(SecondaryButtonStyle())
            }

            Label("Decrypted on your iPhone", systemImage: "lock.fill")
                .font(.system(size: 13))
                .foregroundStyle(Theme.textMuted)
                .multilineTextAlignment(.center)
                .frame(maxWidth: .infinity)
        }
    }
}
