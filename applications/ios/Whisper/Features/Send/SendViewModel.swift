import Foundation
import WhisperCore

@MainActor
@Observable
final class SendViewModel {
    enum Phase: Equatable {
        case composing
        case sealing
        case created(WhisperLink)
    }

    /// The web form's two modes.
    enum Mode: String, CaseIterable, Identifiable {
        case freeForm = "Free Form"
        case multipleValues = "Multiple Values"
        var id: Self { self }
    }

    struct DraftRow: Identifiable {
        let id = UUID()
        var key = ""
        var value = ""
    }

    struct DraftSection: Identifiable {
        let id = UUID()
        var name = ""
        var rows = [DraftRow()]
    }

    var mode: Mode = .freeForm
    var secret = ""
    var rows = [DraftRow()]
    var sections: [DraftSection] = []

    var expiration: SecretExpiration = .oneHour
    /// Backs the "Custom…" option; kept in step with `expiration` while it's chosen.
    var customDate = Date().addingTimeInterval(24 * 60 * 60) {
        didSet { if isCustomExpiration { expiration = .custom(customDate) } }
    }
    var selfDestruct = true
    private(set) var phase: Phase = .composing
    var errorMessage: String?
    /// The Sent-list entry for the link on screen, when the server tracks
    /// delivery (v1.5.0+).
    private(set) var lastSent: SentSecret?

    private let dependencies: Dependencies

    init(dependencies: Dependencies = .live) {
        self.dependencies = dependencies
    }

    var isCustomExpiration: Bool {
        if case .custom = expiration { return true }
        return false
    }

    var byteCount: Int {
        switch mode {
        case .freeForm: return secret.utf8.count
        case .multipleValues: return ((try? structured())?.json.utf8.count) ?? 0
        }
    }

    var isOverLimit: Bool { byteCount > CreateClientEncryptedSecret.maxBytes }

    var canCreate: Bool {
        let hasContent: Bool
        switch mode {
        case .freeForm:
            hasContent = !secret.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
        case .multipleValues:
            hasContent = (rows + sections.flatMap(\.rows)).contains {
                !$0.key.trimmingCharacters(in: .whitespaces).isEmpty && !$0.value.isEmpty
            }
        }
        return hasContent && !isOverLimit && phase != .sealing
    }

    func create(using settings: AppSettings, sent: SentStore) async {
        guard canCreate else { return }
        errorMessage = nil
        phase = .sealing

        do {
            let now = Date()
            let created = try await CreateClientEncryptedSecret(
                plaintext: try plaintext(),
                expiration: expiration,
                selfDestruct: selfDestruct
            ).handle(
                encryption: dependencies.encryption,
                repository: dependencies.repository(settings.server),
                now: now
            )
            clearDrafts()
            lastSent = created.statusToken.map {
                SentSecret(
                    token: $0, server: settings.server, createdAt: now,
                    expiresAt: expiration.date(from: now), selfDestruct: selfDestruct)
            }
            if let lastSent { sent.add(lastSent) }
            phase = .created(created.link)
        } catch {
            errorMessage = error.localizedDescription
            phase = .composing
        }
    }

    func reset() {
        clearDrafts()
        errorMessage = nil
        lastSent = nil
        phase = .composing
    }

    // MARK: Multiple values

    func removeRow(_ id: UUID) {
        rows.removeAll { $0.id == id }
        if rows.isEmpty { rows = [DraftRow()] }
    }

    func removeSection(_ id: UUID) {
        sections.removeAll { $0.id == id }
    }

    // MARK: -

    private func plaintext() throws -> String {
        switch mode {
        case .freeForm: return secret
        case .multipleValues: return try structured().json
        }
    }

    private func structured() throws -> StructuredSecret {
        func convert(_ rows: [DraftRow]) -> [StructuredSecret.Row] {
            rows.map { StructuredSecret.Row(key: $0.key, value: $0.value) }
        }
        return try StructuredSecret(
            rows: convert(rows),
            sections: sections.map { StructuredSecret.Section(name: $0.name, rows: convert($0.rows)) }
        )
    }

    /// Drafts hold secret values; nothing of them should outlive the link.
    private func clearDrafts() {
        secret = ""
        rows = [DraftRow()]
        sections = []
    }
}
