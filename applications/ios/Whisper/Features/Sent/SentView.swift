import SwiftUI

/// Secrets sent from this device and whether each one was opened. No account:
/// the list lives on the device and holds status tokens only.
struct SentView: View {
    @Environment(SentStore.self) private var sent
    @Environment(\.scenePhase) private var scenePhase

    var body: some View {
        NavigationStack {
            Group {
                if sent.entries.isEmpty {
                    emptyState
                } else {
                    list
                }
            }
            .scrollContentBackground(.hidden)
            .background(Theme.backgroundGradient.ignoresSafeArea())
            .navigationTitle("Sent")
        }
        .task { await sent.refreshAll() }
        .onChange(of: scenePhase) { _, phase in
            if phase == .active { Task { await sent.refreshAll() } }
        }
    }

    private var list: some View {
        List {
            Section {
                ForEach(sent.entries) { entry in
                    row(entry)
                }
                .onDelete { offsets in
                    offsets.map { sent.entries[$0].token }.forEach(sent.remove(token:))
                }
            } footer: {
                Text("Saved on this device only — never the secret or its link. Swipe to remove.")
            }
        }
        .refreshable { await sent.refreshAll() }
    }

    private func row(_ entry: SentSecret) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            DeliveryStatusLabel(status: sent.statuses[entry.token])
            HStack(spacing: 6) {
                Text("Sent " + entry.createdAt.formatted(date: .abbreviated, time: .shortened))
                if entry.selfDestruct {
                    Text("· self-destruct")
                }
            }
            .font(.system(size: 13))
            .foregroundStyle(Theme.textMuted)
        }
        .padding(.vertical, 4)
        .accessibilityElement(children: .combine)
        .accessibilityIdentifier("sent-row")
    }

    private var emptyState: some View {
        VStack(spacing: Theme.Spacing.snug) {
            Image(systemName: "tray")
                .font(.system(size: 34))
                .foregroundStyle(Theme.textMuted)
            Text("Nothing sent yet")
                .font(.system(size: 17, weight: .semibold))
                .foregroundStyle(Theme.textPrimary)
            Text("Secrets you create here show up in this list, so you can see when they're opened.")
                .font(.system(size: 14))
                .foregroundStyle(Theme.textMuted)
                .multilineTextAlignment(.center)
        }
        .padding(Theme.Spacing.section)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
