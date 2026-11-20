import SwiftUI
import WhisperCore

/// A "Multiple Values" secret — from the web form, this app, or JSON someone
/// wrote by hand — as labelled fields, each masked until shown and copyable on
/// its own. Or the raw JSON, like the web reveal page's toggle.
struct StructuredSecretCard: View {
    enum Display: String, CaseIterable {
        case fields = "Fields"
        case raw = "Raw JSON"
    }

    let secret: StructuredSecret

    @State private var display: Display = .fields
    @State private var shown: Set<String> = []
    @State private var rawShown = false

    private var allPaths: [String] { Self.paths(secret.entries, prefix: "") }
    private var everythingShown: Bool { shown.count == allPaths.count && rawShown }

    var body: some View {
        VStack(alignment: .leading, spacing: Theme.Spacing.snug) {
            HStack {
                FieldLabel(text: "\(allPaths.count) value\(allPaths.count == 1 ? "" : "s")")
                Spacer()
                Button {
                    withAnimation(.easeOut(duration: 0.3)) {
                        let show = !everythingShown
                        shown = show ? Set(allPaths) : []
                        rawShown = show
                    }
                } label: {
                    Label(
                        everythingShown ? "Hide all" : "Show all",
                        systemImage: everythingShown ? "eye.slash" : "eye"
                    )
                    .font(.system(size: 13, weight: .medium))
                    .foregroundStyle(Theme.accent)
                }
                .accessibilityIdentifier("show-all-toggle")
            }

            ModeSwitch(options: Display.allCases, selection: $display, title: \.rawValue)

            switch display {
            case .fields:
                EntryList(entries: secret.entries, prefix: "", shown: $shown)
            case .raw:
                rawJSON
            }
        }
        .cardBackground()
    }

    private var rawJSON: some View {
        ZStack {
            Text(secret.prettyJSON)
                .font(.system(size: 13, design: .monospaced))
                .foregroundStyle(Theme.textPrimary)
                .textSelection(.enabled)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(Theme.Spacing.snug)
                .background(Theme.input.opacity(0.6), in: RoundedRectangle(cornerRadius: Theme.Radius.medium))
                .blur(radius: rawShown ? 0 : 12)
                .accessibilityHidden(!rawShown)

            if !rawShown {
                Button {
                    withAnimation(.easeOut(duration: 0.3)) { rawShown = true }
                } label: {
                    Label("Tap to reveal", systemImage: "eye.fill")
                        .font(.system(size: 14, weight: .semibold))
                        .foregroundStyle(Theme.textPrimary)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                        .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
            }
        }
    }

    static func paths(_ entries: [StructuredSecret.Entry], prefix: String) -> [String] {
        entries.flatMap { entry -> [String] in
            switch entry {
            case .value(let key, _), .literal(let key, _): return [prefix + key]
            case .section(let name, let nested): return paths(nested, prefix: prefix + name + "/")
            }
        }
    }
}

/// One level of fields; sections recurse into an indented list.
private struct EntryList: View {
    let entries: [StructuredSecret.Entry]
    let prefix: String
    @Binding var shown: Set<String>

    var body: some View {
        VStack(alignment: .leading, spacing: Theme.Spacing.tight) {
            ForEach(Array(entries.enumerated()), id: \.offset) { _, entry in
                switch entry {
                case .value(let key, let value):
                    FieldRow(key: key, value: value, path: prefix + key, shown: $shown)
                case .literal(let key, let json):
                    FieldRow(key: key, value: json, path: prefix + key, shown: $shown)
                case .section(let name, let nested):
                    VStack(alignment: .leading, spacing: Theme.Spacing.tight) {
                        Label(name, systemImage: "folder.fill")
                            .font(.system(size: 14, weight: .semibold))
                            .foregroundStyle(Theme.textPrimary)
                            .symbolRenderingMode(.hierarchical)
                        // Recursion needs a concrete type, hence AnyView.
                        AnyView(EntryList(entries: nested, prefix: prefix + name + "/", shown: $shown))
                            .padding(.leading, Theme.Spacing.snug)
                            .overlay(alignment: .leading) {
                                Capsule().fill(Theme.brandGradient).frame(width: 2)
                            }
                    }
                    .padding(.top, 4)
                }
            }
        }
    }
}

/// `KEY` above a masked value, with show and copy buttons.
private struct FieldRow: View {
    let key: String
    let value: String
    let path: String
    @Binding var shown: Set<String>

    @State private var didCopy = false

    private var isShown: Bool { shown.contains(path) }

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(key)
                .font(.system(size: 12, weight: .semibold, design: .monospaced))
                .foregroundStyle(Theme.textMuted)

            HStack(spacing: Theme.Spacing.snug) {
                Group {
                    if isShown {
                        Text(value)
                            .textSelection(.enabled)
                            .accessibilityIdentifier("field-value-\(path)")
                    } else {
                        // Fixed width, so the mask doesn't give away the length.
                        Text("••••••••••")
                            .accessibilityLabel("Hidden")
                    }
                }
                .font(.system(size: 15, design: .monospaced))
                .foregroundStyle(Theme.textPrimary)
                .frame(maxWidth: .infinity, alignment: .leading)

                Button {
                    withAnimation(.easeOut(duration: 0.2)) {
                        if isShown { shown.remove(path) } else { shown.insert(path) }
                    }
                } label: {
                    Image(systemName: isShown ? "eye.slash" : "eye")
                }
                .accessibilityLabel(isShown ? "Hide \(key)" : "Show \(key)")
                .accessibilityIdentifier("field-toggle-\(path)")

                Button {
                    Clipboard.copyExpiring(value)
                    UINotificationFeedbackGenerator().notificationOccurred(.success)
                    withAnimation { didCopy = true }
                    Task {
                        try? await Task.sleep(for: .seconds(1.5))
                        withAnimation { didCopy = false }
                    }
                } label: {
                    Image(systemName: didCopy ? "checkmark" : "doc.on.doc")
                        .contentTransition(.symbolEffect(.replace))
                }
                .accessibilityLabel("Copy \(key)")
            }
            .foregroundStyle(Theme.accent)
        }
        .padding(Theme.Spacing.snug)
        .background(Theme.input.opacity(0.6), in: RoundedRectangle(cornerRadius: Theme.Radius.medium))
    }
}
