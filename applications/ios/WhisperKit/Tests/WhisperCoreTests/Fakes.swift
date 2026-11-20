import Foundation
import WhisperCore

/// Reversible stand-in for AES-GCM so core tests don't depend on an adapter.
/// The "ciphertext" is the reversed UTF-8 bytes, keyed by a fixed key.
struct ReversingEncryption: SecretEncryption {
    static let key = SecretKey(bytes: Data(repeating: 7, count: 32))!

    func seal(_ plaintext: String) throws -> SealedSecret {
        SealedSecret(key: Self.key, payload: Data(plaintext.utf8.reversed()))
    }

    func open(_ payload: Data, with key: SecretKey) throws -> String {
        guard key == Self.key else { throw SecretEncryptionError.wrongKey }
        return String(decoding: payload.reversed(), as: UTF8.self)
    }
}

/// A server in memory, with burn-after-reading semantics.
actor InMemoryRepository: SharedSecretRepository {
    nonisolated let server = URL(string: "https://whisper.test")!
    private(set) var stored: [SecretId: SharedSecret] = [:]
    var metadataFailure: SharedSecretRepositoryError?

    func insert(_ secret: SharedSecret) -> SecretId {
        let id = SecretId(UUID().uuidString.lowercased())!
        stored[id] = secret
        return id
    }

    func failMetadata(with error: SharedSecretRepositoryError) {
        metadataFailure = error
    }

    /// Off, it behaves like a server before v1.5.0: no status tokens.
    var tracksDelivery = true
    private var receipts: [StatusToken: (id: SecretId, status: DeliveryStatus)] = [:]

    func stopTrackingDelivery() {
        tracksDelivery = false
    }

    func save(payload: Data, expiration: Date, selfDestruct: Bool) async throws -> SavedSecret {
        let id = insert(SharedSecret(content: .clientEncrypted(payload), selfDestruct: selfDestruct))
        guard tracksDelivery else { return SavedSecret(id: id, statusToken: nil) }
        let token = StatusToken(UUID().uuidString.lowercased())!
        receipts[token] = (id, DeliveryStatus(state: .waiting, expiresAt: expiration, openedAt: nil))
        return SavedSecret(id: id, statusToken: token)
    }

    func deliveryStatus(token: StatusToken) async throws -> DeliveryStatus? {
        receipts[token]?.status
    }

    func metadata(id: SecretId) async throws -> SecretMetadata? {
        if let metadataFailure { throw metadataFailure }
        guard let secret = stored[id] else {
            return SecretMetadata(exists: false, clientEncrypted: false, selfDestruct: false)
        }
        if case .clientEncrypted = secret.content {
            return SecretMetadata(exists: true, clientEncrypted: true, selfDestruct: secret.selfDestruct)
        }
        return SecretMetadata(exists: true, clientEncrypted: false, selfDestruct: secret.selfDestruct)
    }

    func fetch(id: SecretId) async throws -> SharedSecret {
        guard let secret = stored[id] else { throw SharedSecretRepositoryError.notFound }
        if secret.selfDestruct { stored[id] = nil }
        // Like the server's receipt: only the first reveal is stamped.
        for (token, receipt) in receipts where receipt.id == id && receipt.status.openedAt == nil {
            receipts[token] = (
                id, DeliveryStatus(state: .opened, expiresAt: receipt.status.expiresAt, openedAt: Date())
            )
        }
        return secret
    }
}
