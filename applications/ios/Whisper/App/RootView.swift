import SwiftUI

struct RootView: View {
    enum Tab: Hashable {
        case send
        case sent
        case receive
        case settings
    }

    @State private var settings = AppSettings()
    @State private var sent = SentStore()
    @State private var receiveModel = ReceiveViewModel()
    @State private var tab: Tab = .send

    var body: some View {
        TabView(selection: $tab) {
            SendView()
                .tabItem { Label("Send", systemImage: "paperplane.fill") }
                .tag(Tab.send)

            SentView()
                .tabItem { Label("Sent", systemImage: "tray.full.fill") }
                .tag(Tab.sent)

            ReceiveView(model: receiveModel)
                .tabItem { Label("Open", systemImage: "envelope.open.fill") }
                .tag(Tab.receive)

            SettingsView()
                .tabItem { Label("Settings", systemImage: "gearshape.fill") }
                .tag(Tab.settings)
        }
        .tint(Theme.accent)
        .background(KeyboardDismissal())
        .environment(settings)
        .environment(sent)
        .onOpenURL { url in
            // A link opened from Messages, Mail or a QR code lands here. It is
            // only inspected — never consumed — so an accidental tap cannot
            // burn a one-time secret.
            tab = .receive
            receiveModel.pasted = url.absoluteString
            Task { await receiveModel.inspect(url.absoluteString, using: settings) }
        }
    }
}
