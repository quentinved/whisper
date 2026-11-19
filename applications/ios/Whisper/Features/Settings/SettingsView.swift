import SwiftUI

struct SettingsView: View {
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
                    Text("Point this at your own Whisper instance to keep secrets on infrastructure you control. Links you open always use the server they came from.")
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
                            ? "Asks for Face ID or your passcode before a secret is shown."
                            : "This device has no passcode or biometrics set up, so this has no effect."
                    )
                }

                Section {
                    LabeledContent("Encryption", value: "AES-256-GCM")
                    LabeledContent("Key location", value: "The link, never the server")
                    LabeledContent("Version", value: Bundle.main.shortVersion)
                } header: {
                    Text("About")
                } footer: {
                    Text("Secrets are encrypted on this iPhone before they are uploaded. The server stores ciphertext and has no key for it, so links made here open in the web app and links from the CLI or Slack open here.")
                }
            }
            .scrollContentBackground(.hidden)
            .background(Theme.backgroundGradient.ignoresSafeArea())
            .navigationTitle("Settings")
        }
    }
}

extension Bundle {
    var shortVersion: String {
        object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String ?? "—"
    }
}
