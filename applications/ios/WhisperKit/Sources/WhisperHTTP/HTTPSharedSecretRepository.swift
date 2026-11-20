import Foundation
import WhisperCore

/// `SharedSecretRepository` over a Whisper server's HTTP API.
/// ≙ `PostgreSQLSharedSecretsRepository` — the persistence adapter, seen from
/// the client. No auth, no session: the server is never trusted with anything
/// it could leak.
public struct HTTPSharedSecretRepository: SharedSecretRepository {
    public let server: URL
    private let session: URLSession

    public init(server: URL, session: URLSession = .shared) {
        self.server = server
        self.session = session
    }

    public func save(payload: Data, expiration: Date, selfDestruct: Bool) async throws -> SavedSecret {
        var request = URLRequest(url: endpoint("v1/ephemeral", source: true))
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = try JSONEncoder().encode(
            CreateBody(
                payload: Base64URL.encode(payload),
                expiration: Int64(expiration.timeIntervalSince1970),
                self_destruct: selfDestruct
            )
        )

        let (data, response) = try await perform(request)
        switch response.statusCode {
        case 200...299:
            guard let decoded = try? JSONDecoder().decode(CreateResponse.self, from: data),
                let id = SecretId(decoded.id)
            else { throw SharedSecretRepositoryError.malformedResponse }
            return SavedSecret(id: id, statusToken: decoded.status_token.flatMap(StatusToken.init))
        case 400, 413:
            throw SharedSecretRepositoryError.tooLarge
        case let code:
            throw SharedSecretRepositoryError.server(code)
        }
    }

    /// A missing route and a missing secret both answer 404, so a body that
    /// isn't metadata means "this server predates `/meta`" → `nil`.
    public func metadata(id: SecretId) async throws -> SecretMetadata? {
        let request = URLRequest(url: endpoint("secret/\(id.rawValue)/meta", source: false))
        let (data, _) = try await perform(request)
        guard let dto = try? JSONDecoder().decode(MetadataBody.self, from: data) else { return nil }
        return SecretMetadata(
            exists: dto.exists,
            clientEncrypted: dto.client_encrypted,
            selfDestruct: dto.self_destruct
        )
    }

    public func fetch(id: SecretId) async throws -> SharedSecret {
        let request = URLRequest(url: endpoint("secret/\(id.rawValue)", source: true))
        let (data, response) = try await perform(request)
        switch response.statusCode {
        case 200...299:
            guard let dto = try? JSONDecoder().decode(SecretBody.self, from: data) else {
                throw SharedSecretRepositoryError.malformedResponse
            }
            return try SharedSecret(content: content(of: dto), selfDestruct: dto.self_destruct)
        case 404:
            throw SharedSecretRepositoryError.notFound
        case let code:
            throw SharedSecretRepositoryError.server(code)
        }
    }

    /// Unknown and purged tokens both answer 404 → `nil`.
    public func deliveryStatus(token: StatusToken) async throws -> DeliveryStatus? {
        var request = URLRequest(url: endpoint("v1/status/\(token.rawValue)", source: false))
        request.cachePolicy = .reloadIgnoringLocalCacheData
        let (data, response) = try await perform(request)
        switch response.statusCode {
        case 200...299:
            guard let dto = try? Self.statusDecoder.decode(StatusBody.self, from: data),
                let state = DeliveryStatus.State(rawValue: dto.state)
            else { throw SharedSecretRepositoryError.malformedResponse }
            return DeliveryStatus(state: state, expiresAt: dto.expires_at, openedAt: dto.opened_at)
        case 404:
            return nil
        case let code:
            throw SharedSecretRepositoryError.server(code)
        }
    }

    // MARK: - Wire format

    private struct CreateBody: Encodable {
        let payload: String
        let expiration: Int64
        let self_destruct: Bool
        /// Opt-in: servers before v1.5.0 ignore it and return no status token.
        let track_delivery = true
    }

    private struct CreateResponse: Decodable {
        let id: String
        let status_token: String?
    }

    /// The server sends whole-second UTC timestamps, which `.iso8601` reads.
    private struct StatusBody: Decodable {
        let state: String
        let expires_at: Date
        let opened_at: Date?
    }

    private static let statusDecoder: JSONDecoder = {
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        return decoder
    }()

    private struct MetadataBody: Decodable {
        let exists: Bool
        let client_encrypted: Bool
        let self_destruct: Bool
    }

    private struct SecretBody: Decodable {
        let secret: String
        let self_destruct: Bool
        let client_encrypted: Bool
    }

    private func content(of dto: SecretBody) throws -> SharedSecret.Content {
        guard dto.client_encrypted else { return .plain(dto.secret) }
        guard let payload = Base64URL.decode(dto.secret) else {
            throw SharedSecretRepositoryError.malformedResponse
        }
        return .clientEncrypted(payload)
    }

    // MARK: - Plumbing

    private func endpoint(_ path: String, source: Bool) -> URL {
        var components = URLComponents(
            url: server.appendingPathComponent(path),
            resolvingAgainstBaseURL: false
        )!
        if source {
            components.queryItems = [URLQueryItem(name: "source", value: "ios")]
        }
        return components.url!
    }

    private func perform(_ request: URLRequest) async throws -> (Data, HTTPURLResponse) {
        let data: Data
        let response: URLResponse
        do {
            (data, response) = try await session.data(for: request)
        } catch {
            throw SharedSecretRepositoryError.offline
        }
        guard let http = response as? HTTPURLResponse else {
            throw SharedSecretRepositoryError.malformedResponse
        }
        return (data, http)
    }
}
