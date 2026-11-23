import SwiftUI

/// "Name this secret" prompt, shared by the Sent list and the "Link ready"
/// card, so a secret can be named after it was shared. The name stays on this
/// device, like the rest of the Sent list.
private struct RenameSentSecret: ViewModifier {
    @Environment(SentStore.self) private var sent
    @Binding var target: SentSecret?
    @State private var draft = ""

    func body(content: Content) -> some View {
        content
            .alert(
                "Name this secret",
                isPresented: Binding(get: { target != nil }, set: { if !$0 { target = nil } })
            ) {
                TextField("e.g. Staging database", text: $draft)
                    .accessibilityIdentifier("rename-field")
                Button("Save") {
                    if let target { sent.rename(token: target.token, to: draft) }
                }
                Button("Cancel", role: .cancel) {}
            } message: {
                Text("Only you see it, on this iPhone. It's never sent to the server.")
            }
            .onChange(of: target) { _, new in draft = new?.name ?? "" }
    }
}

extension View {
    func renameSentSecret(_ target: Binding<SentSecret?>) -> some View {
        modifier(RenameSentSecret(target: target))
    }
}
