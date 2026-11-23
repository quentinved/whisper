import Foundation
import WhisperCore

/// A secret sent from this device. Holds only the status token, which can read
/// whether the secret was opened — never the link, its id or its `#k=` key.
struct SentSecret: Codable, Identifiable, Equatable, Sendable {
    let token: StatusToken
    /// The server it was created on: changing servers in Settings must not
    /// break older entries.
    let server: URL
    let createdAt: Date
    let expiresAt: Date
    let selfDestruct: Bool
    /// What the sender called it. Kept on this device only, never sent to the
    /// server. Optional, so entries saved before names existed still decode.
    let name: String?

    var id: StatusToken { token }

    static let maxNameLength = 60

    /// Trimmed and capped like the web field; empty means no name.
    static func cleanName(_ raw: String) -> String? {
        let trimmed = String(raw.trimmingCharacters(in: .whitespacesAndNewlines).prefix(maxNameLength))
        return trimmed.isEmpty ? nil : trimmed
    }
}

/// The "Sent" list, kept in `UserDefaults` like the web keeps it in
/// `localStorage`. Statuses are fetched live and never persisted.
@MainActor
@Observable
final class SentStore {
    enum RowStatus: Equatable {
        case checking
        case unavailable
        case known(DeliveryStatus)
    }

    static let maxEntries = 20
    private static let key = "sent_secrets"

    private(set) var entries: [SentSecret]
    private(set) var statuses: [StatusToken: RowStatus] = [:]
    private let defaults: UserDefaults
    private let dependencies: Dependencies

    init(defaults: UserDefaults = .standard, dependencies: Dependencies = .live) {
        self.defaults = defaults
        self.dependencies = dependencies
        entries =
            defaults.data(forKey: Self.key)
            .flatMap { try? JSONDecoder().decode([SentSecret].self, from: $0) } ?? []
    }

    func add(_ entry: SentSecret) {
        entries = Array(([entry] + entries).prefix(Self.maxEntries))
        save()
    }

    func remove(token: StatusToken) {
        entries.removeAll { $0.token == token }
        statuses[token] = nil
        save()
    }

    func refreshAll() async {
        for entry in entries {
            await refresh(entry)
        }
    }

    /// A token the server no longer knows (purged 7 days after expiry) drops
    /// out of the list; a network failure keeps it and says so.
    func refresh(_ entry: SentSecret) async {
        if statuses[entry.token] == nil { statuses[entry.token] = .checking }
        do {
            guard
                let status = try await GetDeliveryStatus(token: entry.token)
                    .handle(repository: dependencies.repository(entry.server))
            else {
                remove(token: entry.token)
                return
            }
            statuses[entry.token] = .known(status)
        } catch {
            statuses[entry.token] = .unavailable
        }
    }

    private func save() {
        defaults.set(try? JSONEncoder().encode(entries), forKey: Self.key)
    }
}
