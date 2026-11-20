import Foundation

/// A secret sealed on the device, split into the half that goes to the server
/// and the half that never does. ≙ `SecretEncrypted` in services-core.
public struct SealedSecret: Sendable {
    /// Travels only in the URL fragment.
    public let key: SecretKey
    /// `nonce[12] ‖ ciphertext ‖ tag[16]`. Useless without the key.
    public let payload: Data

    public init(key: SecretKey, payload: Data) {
        self.key = key
        self.payload = payload
    }
}
