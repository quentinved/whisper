import SwiftUI

struct SettingsView: View {
    static let repository = URL(string: "https://github.com/quentinved/whisper")!

    @Environment(AppSettings.self) private var settings

    var body: some View {
        @Bindable var settings = settings

        NavigationStack {
            Form {
                Section {
                    TextField("https://whisper.example.com", text: $settings.serverURLString)
                        .font(.system(size: 15, design: .monospaced))
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                        .keyboardType(.URL)
                        .submitLabel(.done)
                        .accessibilityIdentifier("server-field")
                    if !settings.serverIsValid {
                        Text("Not a valid URL — falling back to the default server.")
                            .font(.system(size: 13))
                            .foregroundStyle(Theme.error)
                    }
                    Button("Reset to default") {
                        settings.serverURLString = AppSettings.defaultServer.absoluteString
                    }
                    .foregroundStyle(Theme.textSecondary)
                } header: {
                    Text("Server")
                } footer: {
                    Text("Use your own Whisper server. Links you open always use the server they came from.")
                }

                Section {
                    Toggle("Require Face ID to reveal", isOn: $settings.requireBiometrics)
                        .tint(Theme.accent)
                        .accessibilityIdentifier("biometrics-toggle")
                } header: {
                    Text("Privacy")
                } footer: {
                    Text(
                        BiometricGate.isAvailable
                            ? "Asks for Face ID or your passcode before showing a secret."
                            : "No Face ID or passcode is set up on this iPhone, so this does nothing."
                    )
                }

                Section {
                    Link(destination: Self.repository) {
                        Label("View the code on GitHub", systemImage: "chevron.left.forwardslash.chevron.right")
                    }
                    Link(destination: Self.repository.appendingPathComponent("issues")) {
                        Label("Report a bug or request a feature", systemImage: "ladybug")
                    }
                } header: {
                    Text("Open source")
                } footer: {
                    Text("Read exactly how your secrets are encrypted, or give the repo a ⭐️.")
                }
                .tint(Theme.accent)

                Section {
                    LabeledContent("Encryption", value: "AES-256-GCM")
                    LabeledContent("Key location", value: "The link, never the server")
                    LabeledContent("Version", value: Bundle.main.shortVersion)
                } header: {
                    Text("About")
                } footer: {
                    Text("Secrets are encrypted on this iPhone. The server only stores data it can't read. Links work across the app, the website, the CLI and Slack.")
                }
            }
            .scrollContentBackground(.hidden)
            .scrollDismissesKeyboard(.immediately)
            .background(AuroraBackground())
            .navigationTitle("Settings")
        }
    }
}

extension Bundle {
    var shortVersion: String {
        object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "—"
    }
}
