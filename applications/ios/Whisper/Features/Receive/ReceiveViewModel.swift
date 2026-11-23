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
        case revealed(String, burned: Bool, until: Date)
        /// Already opened or expired: a calm end state, not an error on the paste form.
        case gone
    }

    /// How long a revealed secret stays on screen before the app forgets it.
    /// UI tests shorten it through the launch environment; absent in any normal run.
    static let revealLifetime: TimeInterval =
        ProcessInfo.processInfo.environment["WHISPER_REVEAL_LIFETIME"].flatMap(TimeInterval.init)
        ?? 120

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
        } catch SharedSecretRepositoryError.notFound {
            showGone()
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
            let until = Date().addingTimeInterval(Self.revealLifetime)
            phase = .revealed(revealed.text, burned: revealed.burned, until: until)
            forget(at: until)
        } catch SharedSecretRepositoryError.notFound {
            showGone()
        } catch {
            errorMessage = error.localizedDescription
            phase = .confirming(link, metadata)
        }
    }

    /// Drops the plaintext once its time is up, unless the user already moved on
    /// to another link — only the reveal that scheduled this is cleared.
    private func forget(at until: Date) {
        Task { [weak self] in
            try? await Task.sleep(for: .seconds(Self.revealLifetime))
            guard let self, case .revealed(_, _, let current) = self.phase, current == until
            else { return }
            self.reset()
        }
    }

    private func showGone() {
        pasted = ""
        errorMessage = nil
        phase = .gone
    }

    func reset() {
        pasted = ""
        errorMessage = nil
        phase = .idle
    }
}
