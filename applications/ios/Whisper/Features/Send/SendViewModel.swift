import Foundation

@MainActor
@Observable
final class SendViewModel {
    enum Phase: Equatable {
        case composing
        case sealing
        case created(WhisperLink)
    }

    var secret = ""
    var expiration: Expiration = .oneHour
    var selfDestruct = true
    private(set) var phase: Phase = .composing
    /// The entry added to the Sent list for the link on screen, if the server
    /// tracks delivery.
    private(set) var lastSent: SentSecret?
    var errorMessage: String?

    /// Matches `MAX_SECRET_SIZE` in
    /// `services-core/src/commands/shared_secret/create_secret.rs`. Measured in
    /// bytes, not characters, because that is what the server counts — an
    /// emoji-heavy secret hits the ceiling four times faster than its length
    /// suggests.
    static let maxBytes = 64 * 1024

    var byteCount: Int { secret.utf8.count }
    var isOverLimit: Bool { byteCount > Self.maxBytes }
    var canCreate: Bool {
        !secret.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
            && !isOverLimit
            && phase != .sealing
    }

    func create(using settings: AppSettings, sent: SentStore) async {
        guard canCreate else { return }
        errorMessage = nil
        phase = .sealing

        do {
            // Sealing happens before anything touches the network, so the
            // plaintext never leaves this process and the key never leaves the
            // device.
            let sealed = try EphemeralCrypto.seal(secret)
            let expiresAt = expiration.date()
            let created = try await settings.client.createEphemeral(
                payloadB64URL: sealed.payloadB64URL,
                expiration: expiresAt,
                selfDestruct: selfDestruct
            )

            let link = WhisperLink(id: created.id, key: sealed.keyB64URL, server: settings.server)
            lastSent = created.statusToken.map {
                SentSecret(
                    token: $0, server: settings.server, createdAt: .now,
                    expiresAt: expiresAt, selfDestruct: selfDestruct)
            }
            if let lastSent { sent.add(lastSent) }
            secret = ""
            phase = .created(link)
        } catch {
            errorMessage = error.localizedDescription
            phase = .composing
        }
    }

    func reset() {
        secret = ""
        lastSent = nil
        errorMessage = nil
        phase = .composing
    }
}
