import Foundation

@MainActor
@Observable
final class ReceiveViewModel {
    enum Phase: Equatable {
        case idle
        case inspecting
        case confirming(WhisperLink, SecretMetadata?)
        case revealing
        case revealed(String, burned: Bool)
    }

    var pasted = ""
    private(set) var phase: Phase = .idle
    var errorMessage: String?

    /// Step one: identify the secret without consuming it.
    ///
    /// Opening a burn-after-reading secret is irreversible, so the app never
    /// fetches on the strength of a tap that only meant "I have a link". It
    /// parses, asks the server what kind of secret this is, and stops.
    func inspect(_ raw: String, using settings: AppSettings) async {
        errorMessage = nil

        guard let link = WhisperLink.parse(raw, fallbackServer: settings.server) else {
            errorMessage =
                "That doesn't look like a Whisper link. Make sure you copied all of it, including the part after #."
            return
        }

        phase = .inspecting
        // A server older than the `/meta` endpoint answers `nil`; the app then
        // warns in the conditional ("may be destroyed") rather than asserting.
        let metadata = try? await settings.client.metadata(id: link.id)

        if let metadata, !metadata.exists {
            phase = .idle
            errorMessage = WhisperClientError.notFound.errorDescription
            return
        }

        phase = .confirming(link, metadata)
    }

    /// Step two, taken only after an explicit confirmation: consume it.
    func reveal(using settings: AppSettings) async {
        guard case .confirming(let link, let metadata) = phase else { return }
        errorMessage = nil

        if settings.requireBiometrics {
            let unlocked = await BiometricGate.authenticate(reason: "Reveal this secret")
            guard unlocked else { return }
        }

        phase = .revealing

        do {
            let fetched = try await settings.client.fetch(id: link.id)

            // A secret created before zero-knowledge sharing existed comes back
            // already decrypted by the server; a modern one is opaque and needs
            // the key from the link.
            let plaintext =
                fetched.clientEncrypted
                ? try EphemeralCrypto.open(payloadB64URL: fetched.secret, keyB64URL: link.key)
                : fetched.secret

            pasted = ""
            phase = .revealed(plaintext, burned: fetched.selfDestruct)
        } catch {
            errorMessage = error.localizedDescription
            phase = .confirming(link, metadata)
        }
    }

    func reset() {
        pasted = ""
        errorMessage = nil
        phase = .idle
    }
}
