import Foundation
import WhisperCore

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

    private let dependencies: Dependencies

    init(dependencies: Dependencies = .live) {
        self.dependencies = dependencies
    }

    /// Step one: identify the secret without consuming it.
    func inspect(_ raw: String, using settings: AppSettings) async {
        errorMessage = nil

        guard let link = WhisperLink(parsing: raw, fallbackServer: settings.server) else {
            errorMessage =
                "That doesn't look like a Whisper link. Make sure you copied all of it, including the part after #."
            return
        }

        phase = .inspecting
        do {
            let metadata = try await GetSecretMetadata(id: link.id)
                .handle(repository: dependencies.repository(link.server))
            phase = .confirming(link, metadata)
        } catch {
            phase = .idle
            errorMessage = error.localizedDescription
        }
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
            let revealed = try await GetSecretById(link: link).handle(
                encryption: dependencies.encryption,
                repository: dependencies.repository(link.server)
            )
            pasted = ""
            phase = .revealed(revealed.text, burned: revealed.burned)
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
