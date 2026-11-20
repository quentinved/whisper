/// Whether a secret sent from this device was opened. Sender-only and
/// non-consuming. ≙ `GetDeliveryReceipt` in services-core.
public struct GetDeliveryStatus: Sendable {
    private let token: StatusToken

    public init(token: StatusToken) {
        self.token = token
    }

    /// `nil` once the server no longer knows the token: receipts are purged 7
    /// days after the secret expires.
    public func handle(repository: some SharedSecretRepository) async throws -> DeliveryStatus? {
        try await repository.deliveryStatus(token: token)
    }
}
