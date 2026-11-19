import Foundation

/// What the server is willing to say about a secret without consuming it.
struct SecretMetadata: Decodable, Sendable, Equatable {
    let exists: Bool
    let clientEncrypted: Bool
    let selfDestruct: Bool

    enum CodingKeys: String, CodingKey {
        case exists
        case clientEncrypted = "client_encrypted"
        case selfDestruct = "self_destruct"
    }
}

/// A secret as returned by `GET /secret/:id` — this call *consumes* a
/// self-destruct secret, so it is only ever made once the user has confirmed.
struct FetchedSecret: Decodable, Sendable {
    let secret: String
    let selfDestruct: Bool
    let clientEncrypted: Bool

    enum CodingKeys: String, CodingKey {
        case secret
        case selfDestruct = "self_destruct"
        case clientEncrypted = "client_encrypted"
    }
}

/// A freshly stored secret. `statusToken` is absent on servers that predate
/// delivery status, in which case the secret is simply not tracked.
struct CreatedSecret: Sendable, Equatable {
    let id: String
    let statusToken: String?
}

/// Whether a sent secret was opened, as told by `GET /v1/status/:token`. The
/// status token can only read this — never the secret itself.
struct DeliveryStatus: Decodable, Sendable, Equatable {
    enum State: String, Decodable, Sendable {
        case waiting
        case opened
        case expired
    }

    let state: State
    let expiresAt: Date
    let openedAt: Date?

    enum CodingKeys: String, CodingKey {
        case state
        case expiresAt = "expires_at"
        case openedAt = "opened_at"
    }
}

enum WhisperClientError: LocalizedError, Equatable {
    case notFound
    case tooLarge
    case offline
    case server(Int)
    case malformedResponse

    var errorDescription: String? {
        switch self {
        case .notFound:
            return "This secret no longer exists. It may have expired, or someone opened it first."
        case .tooLarge:
            return "That secret is too large. The limit is 64 KB."
        case .offline:
            return "Can't reach the server. Check your connection."
        case .server(let code):
            return "The server returned an error (\(code))."
        case .malformedResponse:
            return "The server sent a response this app couldn't read."
        }
    }
}

/// Talks to a Whisper instance. Deliberately tiny: three calls, no auth, no
/// session — the security model is that the server is never trusted with
/// anything it could leak.
struct WhisperClient: Sendable {
    let server: URL
    var session: URLSession = .shared

    private static let decoder = JSONDecoder()
    private static let statusDecoder: JSONDecoder = {
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        return decoder
    }()

    /// Stores a client-encrypted payload and returns its id, plus a status
    /// token when the server tracks delivery.
    func createEphemeral(
        payloadB64URL: String,
        expiration: Date,
        selfDestruct: Bool
    ) async throws -> CreatedSecret {
        struct Body: Encodable {
            let payload: String
            let expiration: Int64
            let self_destruct: Bool
            let track_delivery = true
        }
        struct Response: Decodable {
            let id: String
            let status_token: String?
        }

        var request = URLRequest(url: endpoint("v1/ephemeral", source: true))
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = try JSONEncoder().encode(
            Body(
                payload: payloadB64URL,
                expiration: Int64(expiration.timeIntervalSince1970),
                self_destruct: selfDestruct
            )
        )

        let (data, response) = try await perform(request)
        switch response.statusCode {
        case 200...299:
            guard let decoded = try? Self.decoder.decode(Response.self, from: data) else {
                throw WhisperClientError.malformedResponse
            }
            return CreatedSecret(id: decoded.id, statusToken: decoded.status_token)
        case 400, 413:
            throw WhisperClientError.tooLarge
        case let code:
            throw WhisperClientError.server(code)
        }
    }

    /// Non-consuming peek, used to warn before burning a one-time secret.
    ///
    /// Returns `nil` when the instance predates the `/meta` endpoint, since a
    /// missing route and a missing secret both answer 404. Callers must treat
    /// `nil` as "unknown", not as "safe" — the only difference it makes is
    /// whether the warning can name the secret's exact behaviour.
    func metadata(id: String) async throws -> SecretMetadata? {
        var request = URLRequest(url: endpoint("secret/\(id)/meta", source: false))
        request.httpMethod = "GET"

        let (data, _) = try await perform(request)
        return try? Self.decoder.decode(SecretMetadata.self, from: data)
    }

    /// Consuming read. For a self-destruct secret this is irreversible.
    func fetch(id: String) async throws -> FetchedSecret {
        var request = URLRequest(url: endpoint("secret/\(id)", source: true))
        request.httpMethod = "GET"

        let (data, response) = try await perform(request)
        switch response.statusCode {
        case 200...299:
            guard let decoded = try? Self.decoder.decode(FetchedSecret.self, from: data) else {
                throw WhisperClientError.malformedResponse
            }
            return decoded
        case 404:
            throw WhisperClientError.notFound
        case let code:
            throw WhisperClientError.server(code)
        }
    }

    /// Sender-only, non-consuming. `nil` once the server no longer knows the
    /// token (receipts are purged 7 days after the secret expires).
    func deliveryStatus(token: String) async throws -> DeliveryStatus? {
        var request = URLRequest(url: endpoint("v1/status/\(token)", source: false))
        request.httpMethod = "GET"
        request.cachePolicy = .reloadIgnoringLocalCacheData

        let (data, response) = try await perform(request)
        switch response.statusCode {
        case 200...299:
            guard let decoded = try? Self.statusDecoder.decode(DeliveryStatus.self, from: data)
            else {
                throw WhisperClientError.malformedResponse
            }
            return decoded
        case 404:
            return nil
        case let code:
            throw WhisperClientError.server(code)
        }
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
        do {
            let (data, response) = try await session.data(for: request)
            guard let http = response as? HTTPURLResponse else {
                throw WhisperClientError.malformedResponse
            }
            return (data, http)
        } catch let error as WhisperClientError {
            throw error
        } catch {
            throw WhisperClientError.offline
        }
    }
}
