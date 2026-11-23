import SwiftUI
import WhisperCore

struct SendView: View {
    @Environment(AppSettings.self) private var settings
    @Environment(SentStore.self) private var sent
    @State private var model = SendViewModel()
    @FocusState private var editorFocused: Bool

    var body: some View {
        Screen {
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
                    .transition(
                        .asymmetric(
                            insertion: .scale(scale: 0.94).combined(with: .opacity),
                            removal: .opacity
                        ))
                case .composing, .sealing:
                    composer
                        .transition(.opacity)
                }
            }
        }
        .animation(.spring(response: 0.4, dampingFraction: 0.85), value: model.phase)
    }

    // MARK: - Composer

    private var composer: some View {
        VStack(alignment: .leading, spacing: Theme.Spacing.loose) {
            header

            VStack(alignment: .leading, spacing: Theme.Spacing.snug) {
                ModeSwitch(
                    options: SendViewModel.Mode.allCases, selection: $model.mode, title: \.rawValue)
                switch model.mode {
                case .freeForm: secretEditor
                case .multipleValues: MultipleValuesEditor(model: model)
                }
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

            Label("End-to-end encrypted on your iPhone", systemImage: "lock.fill")
                .font(.system(size: 13))
                .foregroundStyle(Theme.textMuted)
                .frame(maxWidth: .infinity)
        }
    }

    private var header: some View {
        VStack(spacing: Theme.Spacing.tight) {
            GlassBadge(size: 76, cornerRadius: 24) {
                BrandMark(size: 40)
            }
            .accessibilityLabel("Whisper")

            Text("Whisper")
                .font(.system(size: 32, weight: .bold))
                .tracking(-0.6)
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
        .background(Theme.input.opacity(0.85))
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
        .shadow(color: Theme.violet.opacity(editorFocused ? 0.25 : 0), radius: 12)
        .animation(.easeOut(duration: 0.15), value: editorFocused)
    }

    private var optionsCard: some View {
        VStack(spacing: 0) {
            VStack(alignment: .leading, spacing: 6) {
                optionLabel("Name", detail: "Optional · only you see it, in your Sent list")
                TextField("e.g. Staging database", text: $model.label)
                    .font(.system(size: 16))
                    .foregroundStyle(Theme.textPrimary)
                    .textInputAutocapitalization(.sentences)
                    .submitLabel(.done)
                    .onChange(of: model.label) { _, new in
                        if new.count > SentSecret.maxNameLength {
                            model.label = String(new.prefix(SentSecret.maxNameLength))
                        }
                    }
                    .accessibilityIdentifier("name-field")
            }
            .padding(Theme.Spacing.normal)

            Divider().overlay(Theme.border)

            HStack {
                optionLabel("Expires after", detail: "Deleted automatically, opened or not")
                Spacer()
                Picker("Expires after", selection: $model.expiration) {
                    ForEach(SecretExpiration.presets, id: \.self) { option in
                        Text(option.label).tag(option)
                    }
                    Text("Custom…").tag(SecretExpiration.custom(model.customDate))
                }
                .pickerStyle(.menu)
                .tint(Theme.accent)
                .accessibilityIdentifier("expiration-picker")
            }
            .padding(Theme.Spacing.normal)

            if model.isCustomExpiration {
                DatePicker(
                    "Delete on",
                    selection: $model.customDate,
                    in: Date().addingTimeInterval(60)...Date().addingTimeInterval(SecretExpiration.maximum),
                    displayedComponents: [.date, .hourAndMinute]
                )
                .font(.system(size: 16))
                .foregroundStyle(Theme.textPrimary)
                .tint(Theme.accent)
                .padding(.horizontal, Theme.Spacing.normal)
                .padding(.bottom, Theme.Spacing.normal)
                .transition(.opacity.combined(with: .move(edge: .top)))
            }

            Divider().overlay(Theme.border)

            Toggle(isOn: $model.selfDestruct) {
                optionLabel("Burn after reading", detail: "Destroyed the moment it's opened")
            }
            .tint(Theme.accent)
            .padding(Theme.Spacing.normal)
        }
        .glassPanel()
        .animation(.spring(response: 0.35, dampingFraction: 0.85), value: model.isCustomExpiration)
    }

    private func optionLabel(_ title: String, detail: String) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title)
                .font(.system(size: 16))
                .foregroundStyle(Theme.textPrimary)
            Text(detail)
                .font(.system(size: 13))
                .foregroundStyle(Theme.textMuted)
        }
    }

    private var createButton: some View {
        Button {
            dismissKeyboard()
            UIImpactFeedbackGenerator(style: .medium).impactOccurred()
            Task { await model.create(using: settings, sent: sent) }
        } label: {
            if model.phase == .sealing {
                HStack(spacing: Theme.Spacing.tight) {
                    ProgressView().tint(Theme.onBrand)
                    Text("Encrypting…")
                }
            } else {
                Label("Create secret link", systemImage: "link")
            }
        }
        .buttonStyle(PrimaryButtonStyle())
        .disabled(!model.canCreate)
        .accessibilityIdentifier("create-button")
    }
}

/// The app icon's ribbon, in its violet-to-mint gradient with a soft glow.
struct BrandMark: View {
    var size: CGFloat

    var body: some View {
        Image("Mark")
            .renderingMode(.template)
            .resizable()
            .scaledToFit()
            .frame(width: size, height: size)
            .foregroundStyle(Theme.markGradient)
            .shadow(color: Theme.violet.opacity(0.35), radius: size / 6)
    }
}
