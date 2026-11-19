import Foundation

/// A secret sent from this device. Holds only the status token, which can read
/// whether the secret was opened — never the link, its id or its `#k=` key.
struct SentSecret: Codable, Identifiable, Equatable, Sendable {
    let token: String
    /// The server it was created on: changing servers in Settings must not
    /// break older entries.
    let server: URL
    let createdAt: Date
    let expiresAt: Date
    let selfDestruct: Bool

    var id: String { token }
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
    private(set) var statuses: [String: RowStatus] = [:]
    private let defaults: UserDefaults

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
        entries =
            defaults.data(forKey: Self.key)
            .flatMap { try? JSONDecoder().decode([SentSecret].self, from: $0) } ?? []
    }

    func add(_ entry: SentSecret) {
        entries = Array(([entry] + entries).prefix(Self.maxEntries))
        save()
    }

    func remove(token: String) {
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
                let status = try await WhisperClient(server: entry.server)
                    .deliveryStatus(token: entry.token)
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
