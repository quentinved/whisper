import Foundation
import SwiftUI

/// User-visible configuration. Small on purpose: an account-less app has very
/// little to remember, and anything it did remember would be a liability.
@Observable
final class AppSettings {
    static let defaultServer = URL(string: "https://whisper.quentinvedrenne.com")!

    var serverURLString: String {
        didSet { UserDefaults.standard.set(serverURLString, forKey: Keys.server) }
    }

    /// Gate revealing a secret behind Face ID / passcode. Silently skipped on
    /// devices with no biometry and no passcode, so it can default to on.
    var requireBiometrics: Bool {
        didSet { UserDefaults.standard.set(requireBiometrics, forKey: Keys.biometrics) }
    }

    private enum Keys {
        static let server = "server_url"
        static let biometrics = "require_biometrics"
    }

    /// Aims the app at a throwaway server for the end-to-end script in
    /// `applications/ios/Tools/e2e_check.sh`. Passed as an environment variable
    /// at launch, so it is absent from any normal run and is never persisted.
    private static var launchOverride: String? {
        ProcessInfo.processInfo.environment["WHISPER_SERVER"]
    }

    init() {
        // Assigning in `init` deliberately skips `didSet`, so an override used
        // for testing does not survive into the user's saved settings.
        serverURLString =
            Self.launchOverride
            ?? UserDefaults.standard.string(forKey: Keys.server)
            ?? Self.defaultServer.absoluteString
        // The Face ID gate cannot be satisfied by an automated test on a
        // simulator with biometry enrolled, so UI tests switch it off the same
        // way they aim the app at a local server. Absent in any normal run.
        requireBiometrics =
            ProcessInfo.processInfo.environment["WHISPER_DISABLE_BIOMETRIC_GATE"] == "1"
            ? false
            : (UserDefaults.standard.object(forKey: Keys.biometrics) as? Bool ?? true)
    }

    /// Falls back to the default rather than failing, so a typo in settings
    /// can't brick the composer.
    var server: URL {
        guard let url = URL(string: serverURLString.trimmingCharacters(in: .whitespaces)),
            url.scheme == "https" || url.scheme == "http",
            url.host != nil
        else { return Self.defaultServer }
        return url
    }

    var serverIsValid: Bool {
        URL(string: serverURLString.trimmingCharacters(in: .whitespaces))?.host != nil
    }

    var client: WhisperClient { WhisperClient(server: server) }
}
