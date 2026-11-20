import Foundation
import WhisperCore
import WhisperCrypto
import WhisperHTTP

/// Composition root: the only file in the app that names concrete adapters.
/// ≙ the adapter wiring in `applications/axum/server/src/main.rs`.
struct Dependencies: Sendable {
    var encryption: any SecretEncryption = AesGcm()
    /// Per server, because a link is always opened against the server it came
    /// from, which may not be the one configured in Settings.
    var repository: @Sendable (URL) -> any SharedSecretRepository = {
        HTTPSharedSecretRepository(server: $0)
    }

    static let live = Dependencies()
}
