import SwiftUI

/// Secrets sent from this device and whether each one was opened. No account:
/// the list lives on the device and holds status tokens only.
struct SentView: View {
    @Environment(SentStore.self) private var sent
    @Environment(\.scenePhase) private var scenePhase
    @State private var renaming: SentSecret?

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
            .background(AuroraBackground())
            .navigationTitle("Sent")
        }
        .task { await sent.refreshAll() }
        .onChange(of: scenePhase) { _, phase in
            if phase == .active { Task { await sent.refreshAll() } }
        }
        .renameSentSecret($renaming)
    }

    private var list: some View {
        List {
            Section {
                ForEach(sent.entries) { entry in
                    row(entry)
                        .swipeActions(edge: .leading) {
                            Button(entry.name == nil ? "Name" : "Rename") { renaming = entry }
                                .tint(Theme.accent)
                        }
                }
                .onDelete { offsets in
                    offsets.map { sent.entries[$0].token }.forEach(sent.remove(token:))
                }
            } footer: {
                Text("Kept on this iPhone only, never the secret or its link. Tap a secret to name it; names are only visible to you. Swipe left to remove.")
            }
        }
        .refreshable { await sent.refreshAll() }
    }

    /// Tapping a row names (or renames) it — the way to label a secret after
    /// it was shared.
    private func row(_ entry: SentSecret) -> some View {
        Button { renaming = entry } label: { rowContent(entry) }
            .buttonStyle(.plain)
            .accessibilityHint(entry.name == nil ? "Add a name only you can see" : "Rename")
            .accessibilityIdentifier("sent-row")
    }

    private func rowContent(_ entry: SentSecret) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            if let name = entry.name {
                Text(name)
                    .font(.system(size: 16, weight: .semibold))
                    .foregroundStyle(Theme.textPrimary)
                    .lineLimit(1)
            } else {
                // Not a `Label`: in a List it insets the row separator and
                // spaces the icon like a leading accessory.
                HStack(spacing: 5) {
                    Image(systemName: "pencil")
                    Text("Add a name")
                }
                .font(.system(size: 13, weight: .medium))
                .foregroundStyle(Theme.accent)
            }
            DeliveryStatusLabel(status: sent.statuses[entry.token], selfDestruct: entry.selfDestruct)
            Text(
                "Sent " + entry.createdAt.formatted(date: .abbreviated, time: .shortened)
                    + (entry.selfDestruct && !isSettled(entry) ? " · opens once" : "")
            )
            .font(.system(size: 13))
            .foregroundStyle(Theme.textMuted)
        }
        .padding(.vertical, 4)
        .frame(maxWidth: .infinity, alignment: .leading)
        .contentShape(Rectangle())
        .accessibilityElement(children: .combine)
    }

    /// Once opened or expired, "opens once" says nothing the status doesn't.
    private func isSettled(_ entry: SentSecret) -> Bool {
        guard case .known(let status) = sent.statuses[entry.token] else { return false }
        return status.state != .waiting
    }

    private var emptyState: some View {
        VStack(spacing: Theme.Spacing.snug) {
            GlassBadge(size: 76, cornerRadius: 24) {
                Image(systemName: "tray")
                    .font(.system(size: 28, weight: .medium))
                    .foregroundStyle(Theme.markGradient)
            }
            Text("Nothing sent yet")
                .font(.system(size: 20, weight: .bold))
                .foregroundStyle(Theme.textPrimary)
            Text("Secrets you send show up here, so you can see when they're opened.")
                .font(.system(size: 15))
                .foregroundStyle(Theme.textMuted)
                .multilineTextAlignment(.center)
        }
        .padding(Theme.Spacing.section)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
