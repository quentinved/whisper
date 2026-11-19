import SwiftUI

struct SendView: View {
    @Environment(AppSettings.self) private var settings
    @Environment(SentStore.self) private var sent
    @State private var model = SendViewModel()
    @FocusState private var editorFocused: Bool

    var body: some View {
        ScrollView {
            VStack(spacing: Theme.Spacing.loose) {
                switch model.phase {
                case .created(let link):
                    LinkResultView(
                        link: link,
                        expiration: model.expiration,
                        selfDestruct: model.selfDestruct,
                        sentSecret: model.lastSent,
                        onCreateAnother: { withAnimation(.spring(response: 0.4)) { model.reset() } }
                    )
                    .transition(.asymmetric(
                        insertion: .scale(scale: 0.94).combined(with: .opacity),
                        removal: .opacity
                    ))
                case .composing, .sealing:
                    composer
                        .transition(.opacity)
                }
            }
            .padding(Theme.Spacing.normal)
            .padding(.bottom, Theme.Spacing.section)
        }
        .scrollDismissesKeyboard(.interactively)
        .background(Theme.backgroundGradient.ignoresSafeArea())
        .animation(.spring(response: 0.4, dampingFraction: 0.85), value: model.phase)
    }

    // MARK: - Composer

    private var composer: some View {
        VStack(alignment: .leading, spacing: Theme.Spacing.loose) {
            header

            VStack(alignment: .leading, spacing: Theme.Spacing.tight) {
                FieldLabel(text: "Secret")
                secretEditor
                if model.isOverLimit {
                    Text("\(model.byteCount.formatted()) bytes — the limit is 64 KB")
                        .font(.system(size: 12))
                        .foregroundStyle(Theme.error)
                }
            }

            optionsCard

            if let message = model.errorMessage {
                Banner(kind: .error, message: message)
            }

            createButton

            Text("Encrypted on this iPhone before it is sent. The server stores ciphertext it has no key for.")
                .font(.system(size: 13))
                .foregroundStyle(Theme.textMuted)
                .frame(maxWidth: .infinity, alignment: .center)
                .multilineTextAlignment(.center)
                .padding(.horizontal, Theme.Spacing.normal)
        }
    }

    private var header: some View {
        VStack(spacing: Theme.Spacing.tight) {
            // The website's mark, drawn as a template so it inverts in dark
            // mode exactly as `[data-theme="dark"] .logo-image { filter:
            // invert(1) }` does on the web. The glyph is a knockout, so the
            // glass behind it shows through.
            GlassBadge {
                Image("Logo")
                    .renderingMode(.template)
                    .resizable()
                    .scaledToFit()
                    .frame(width: 30, height: 30)
                    .foregroundStyle(Theme.textPrimary)
            }
            .accessibilityLabel("Whisper")

            Text("Whisper")
                .font(.system(size: 30, weight: .bold))
                .tracking(-0.5)
                .foregroundStyle(Theme.textPrimary)

            Text("Built to Disappear.")
                .font(.system(size: 15))
                .foregroundStyle(Theme.textMuted)
        }
        .frame(maxWidth: .infinity)
        .padding(.top, Theme.Spacing.tight)
    }

    private var secretEditor: some View {
        ZStack(alignment: .topLeading) {
            if model.secret.isEmpty {
                Text("Password, API key, private note…")
                    .font(.system(size: 16))
                    .foregroundStyle(Theme.textMuted)
                    .padding(.horizontal, Theme.Spacing.normal)
                    .padding(.vertical, Theme.Spacing.normal + 2)
                    .allowsHitTesting(false)
            }

            TextEditor(text: $model.secret)
                .font(.system(size: 16, design: .monospaced))
                .foregroundStyle(Theme.textPrimary)
                .scrollContentBackground(.hidden)
                .padding(.horizontal, Theme.Spacing.snug)
                .padding(.vertical, Theme.Spacing.snug)
                .frame(minHeight: 148)
                .focused($editorFocused)
                .accessibilityIdentifier("secret-editor")
        }
        .background(Theme.input)
        .clipShape(RoundedRectangle(cornerRadius: Theme.Radius.medium, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: Theme.Radius.medium, style: .continuous)
                .stroke(
                    model.isOverLimit
                        ? Theme.error
                        : (editorFocused ? Theme.borderFocus : Theme.border),
                    lineWidth: editorFocused || model.isOverLimit ? 2 : 1
                )
        )
        .animation(.easeOut(duration: 0.15), value: editorFocused)
    }

    private var optionsCard: some View {
        VStack(spacing: 0) {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text("Expires after")
                        .font(.system(size: 16))
                        .foregroundStyle(Theme.textPrimary)
                    Text("Deleted automatically, opened or not")
                        .font(.system(size: 13))
                        .foregroundStyle(Theme.textMuted)
                }
                Spacer()
                Picker("Expires after", selection: $model.expiration) {
                    ForEach(Expiration.allCases) { option in
                        Text(option.label).tag(option)
                    }
                }
                .pickerStyle(.menu)
                .tint(Theme.textPrimary)
            }
            .padding(Theme.Spacing.normal)

            Divider().overlay(Theme.border)

            Toggle(isOn: $model.selfDestruct) {
                VStack(alignment: .leading, spacing: 2) {
                    Text("Burn after reading")
                        .font(.system(size: 16))
                        .foregroundStyle(Theme.textPrimary)
                    Text("Destroyed the moment it is opened once")
                        .font(.system(size: 13))
                        .foregroundStyle(Theme.textMuted)
                }
            }
            .tint(Theme.accent)
            .padding(Theme.Spacing.normal)
        }
        .background(Theme.card)
        .clipShape(RoundedRectangle(cornerRadius: Theme.Radius.large, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: Theme.Radius.large, style: .continuous)
                .stroke(Theme.border, lineWidth: 1)
        )
    }

    private var createButton: some View {
        Button {
            editorFocused = false
            UIImpactFeedbackGenerator(style: .medium).impactOccurred()
            Task { await model.create(using: settings, sent: sent) }
        } label: {
            if model.phase == .sealing {
                HStack(spacing: Theme.Spacing.tight) {
                    ProgressView().tint(Theme.textInverse)
                    Text("Encrypting…")
                }
            } else {
                Text("Create secret link")
            }
        }
        .buttonStyle(PrimaryButtonStyle())
        .disabled(!model.canCreate)
        .accessibilityIdentifier("create-button")
    }
}
