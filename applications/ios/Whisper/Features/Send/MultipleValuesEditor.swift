import SwiftUI

/// The web's "Multiple Values" builder: key/value rows, plus named sections of
/// rows. Sent as a JSON object, so the web, the CLI and this app all show it
/// as labelled fields.
struct MultipleValuesEditor: View {
    @Bindable var model: SendViewModel

    var body: some View {
        VStack(alignment: .leading, spacing: Theme.Spacing.snug) {
            rowList($model.rows, idPrefix: "kv", onRemove: model.removeRow)
            addButton("Add value", systemImage: "plus") {
                model.rows.append(SendViewModel.DraftRow())
            }

            ForEach($model.sections) { $section in
                sectionEditor($section)
                    .transition(.opacity.combined(with: .scale(scale: 0.97)))
            }

            addButton("Add section", systemImage: "square.stack.3d.up") {
                model.sections.append(SendViewModel.DraftSection())
            }
        }
        .animation(.spring(response: 0.35, dampingFraction: 0.85), value: model.sections.map(\.id))
        .animation(.spring(response: 0.35, dampingFraction: 0.85), value: model.rows.map(\.id))
    }

    private func rowList(
        _ rows: Binding<[SendViewModel.DraftRow]>, idPrefix: String, onRemove: @escaping (UUID) -> Void
    ) -> some View {
        VStack(spacing: Theme.Spacing.tight) {
            // Id-based bindings, so deleting a row can't leave a stale index behind.
            ForEach(rows) { $row in
                let index = rows.wrappedValue.firstIndex { $0.id == row.id } ?? 0
                KeyValueRow(
                    row: $row,
                    canRemove: rows.wrappedValue.count > 1,
                    identifier: "\(idPrefix)-\(index)",
                    onRemove: { onRemove(row.id) }
                )
            }
        }
    }

    private func sectionEditor(_ section: Binding<SendViewModel.DraftSection>) -> some View {
        VStack(alignment: .leading, spacing: Theme.Spacing.tight) {
            HStack {
                Image(systemName: "folder.fill")
                    .foregroundStyle(Theme.markGradient)
                TextField("Section name, e.g. database", text: section.name)
                    .font(.system(size: 15, weight: .semibold))
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                Button {
                    model.removeSection(section.wrappedValue.id)
                } label: {
                    Image(systemName: "trash")
                        .foregroundStyle(Theme.textMuted)
                }
                .accessibilityLabel("Remove section")
            }

            rowList(
                section.rows, idPrefix: "section-kv",
                onRemove: { id in
                    section.wrappedValue.rows.removeAll { $0.id == id }
                    if section.wrappedValue.rows.isEmpty {
                        section.wrappedValue.rows = [SendViewModel.DraftRow()]
                    }
                })

            addButton("Add value", systemImage: "plus") {
                section.wrappedValue.rows.append(SendViewModel.DraftRow())
            }
        }
        .padding(Theme.Spacing.snug)
        .glassPanel()
    }

    private func addButton(_ title: String, systemImage: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Label(title, systemImage: systemImage)
                .font(.system(size: 14, weight: .semibold))
                .foregroundStyle(Theme.accent)
        }
    }
}

/// One `KEY  value` line of the builder.
private struct KeyValueRow: View {
    @Binding var row: SendViewModel.DraftRow
    let canRemove: Bool
    let identifier: String
    let onRemove: () -> Void

    var body: some View {
        HStack(spacing: Theme.Spacing.tight) {
            TextField("KEY", text: $row.key)
                .font(.system(size: 14, weight: .semibold, design: .monospaced))
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .frame(maxWidth: 120)
                .accessibilityIdentifier("\(identifier)-key")

            Divider().frame(height: 22).overlay(Theme.border)

            TextField("value", text: $row.value)
                .font(.system(size: 14, design: .monospaced))
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .accessibilityIdentifier("\(identifier)-value")

            if canRemove {
                Button(action: onRemove) {
                    Image(systemName: "minus.circle.fill")
                        .foregroundStyle(Theme.textMuted)
                }
                .accessibilityLabel("Remove value")
            }
        }
        .foregroundStyle(Theme.textPrimary)
        .padding(.horizontal, Theme.Spacing.snug)
        .frame(height: 48)
        .background(Theme.input.opacity(0.85))
        .clipShape(RoundedRectangle(cornerRadius: Theme.Radius.medium, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: Theme.Radius.medium, style: .continuous)
                .stroke(Theme.border, lineWidth: 1)
        )
    }
}
