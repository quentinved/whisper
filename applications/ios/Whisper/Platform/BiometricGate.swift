import LocalAuthentication

/// Face ID / Touch ID / passcode in front of a revealed secret.
///
/// Deliberately permissive: if the device has no biometry *and* no passcode
/// there is nothing to authenticate against, so the gate opens rather than
/// locking the user out of their own secret. The gate is a shoulder-surfing
/// defence, not part of the cryptographic story.
enum BiometricGate {
    static func authenticate(reason: String) async -> Bool {
        let context = LAContext()
        context.localizedCancelTitle = "Cancel"

        var error: NSError?
        guard context.canEvaluatePolicy(.deviceOwnerAuthentication, error: &error) else {
            return true
        }

        return await withCheckedContinuation { continuation in
            context.evaluatePolicy(.deviceOwnerAuthentication, localizedReason: reason) {
                success, _ in
                continuation.resume(returning: success)
            }
        }
    }

    /// Whether a gate would actually challenge the user, used to avoid
    /// promising Face ID in the UI on a device that has none.
    static var isAvailable: Bool {
        LAContext().canEvaluatePolicy(.deviceOwnerAuthentication, error: nil)
    }
}
