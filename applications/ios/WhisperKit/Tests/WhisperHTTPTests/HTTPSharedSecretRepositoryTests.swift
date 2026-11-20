import Foundation
import Testing
import WhisperCore
import WhisperHTTP

/// Answers every request from `handler`, recording what was sent.
final class StubProtocol: URLProtocol, @unchecked Sendable {
    nonisolated(unsafe) static var handler: (URLRequest) -> (Int, String) = { _ in (500, "") }
    nonisolated(unsafe) static var lastRequest: URLRequest?
    nonisolated(unsafe) static var lastBody: Data?

    override class func canInit(with request: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }

    override func startLoading() {
        Self.lastRequest = request
        Self.lastBody = request.httpBody ?? request.httpBodyStream.map(Self.read)
        let (status, body) = Self.handler(request)
        let response = HTTPURLResponse(url: request.url!, statusCode: status, httpVersion: nil, headerFields: nil)!
        client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
        client?.urlProtocol(self, didLoad: Data(body.utf8))
        client?.urlProtocolDidFinishLoading(self)
    }

    override func stopLoading() {}

    private static func read(_ stream: InputStream) -> Data {
        stream.open()
        defer { stream.close() }
        var data = Data()
        var buffer = [UInt8](repeating: 0, count: 4096)
        while stream.hasBytesAvailable {
            let count = stream.read(&buffer, maxLength: buffer.count)
            guard count > 0 else { break }
            data.append(buffer, count: count)
        }
        return data
    }
}

@Suite(.serialized) struct HTTPSharedSecretRepositoryTests {
    static let id = "3f2b8c1e-9d4a-4b7e-a1c2-5e6f7a8b9c0d"
    let repository: HTTPSharedSecretRepository = {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [StubProtocol.self]
        return HTTPSharedSecretRepository(
            server: URL(string: "https://whisper.test")!,
            session: URLSession(configuration: configuration)
        )
    }()

    static let token = "0a1b2c3d-4e5f-4a6b-8c7d-9e0f1a2b3c4d"

    @Test func savePostsTheBase64URLPayloadAndAsksForDeliveryTracking() async throws {
        StubProtocol.handler = { _ in (200, #"{"id":"\#(Self.id)","status_token":"\#(Self.token)"}"#) }

        let saved = try await repository.save(
            payload: Data([0xFB, 0xFF]), expiration: Date(timeIntervalSince1970: 1_700_000_000), selfDestruct: true
        )

        #expect(saved.id.rawValue == Self.id)
        #expect(saved.statusToken?.rawValue == Self.token)
        let request = try #require(StubProtocol.lastRequest)
        #expect(request.httpMethod == "POST")
        #expect(request.url?.absoluteString == "https://whisper.test/v1/ephemeral?source=ios")
        let body = try #require(StubProtocol.lastBody)
        let json = try #require(try JSONSerialization.jsonObject(with: body) as? [String: Any])
        #expect(json["payload"] as? String == "-_8")
        #expect(json["expiration"] as? Int == 1_700_000_000)
        #expect(json["self_destruct"] as? Bool == true)
        #expect(json["track_delivery"] as? Bool == true)
    }

    @Test func aServerWithoutDeliveryStatusStillSaves() async throws {
        StubProtocol.handler = { _ in (200, #"{"id":"\#(Self.id)"}"#) }

        let saved = try await repository.save(payload: Data([1]), expiration: Date(), selfDestruct: false)

        #expect(saved == SavedSecret(id: SecretId(Self.id)!, statusToken: nil))
    }

    @Test func deliveryStatusReadsWhenTheSecretWasOpened() async throws {
        StubProtocol.handler = { _ in
            (200, #"{"state":"opened","expires_at":"2026-11-20T10:00:00Z","opened_at":"2026-11-20T09:30:00Z"}"#)
        }

        let status = try await repository.deliveryStatus(token: StatusToken(Self.token)!)

        #expect(status?.state == .opened)
        #expect(status?.openedAt == ISO8601DateFormatter().date(from: "2026-11-20T09:30:00Z"))
        #expect(StubProtocol.lastRequest?.url?.path == "/v1/status/\(Self.token)")
    }

    @Test func deliveryStatusOfAPurgedTokenIsNil() async throws {
        StubProtocol.handler = { _ in (404, "") }
        #expect(try await repository.deliveryStatus(token: StatusToken(Self.token)!) == nil)
    }

    @Test func saveMapsARejectedPayloadToTooLarge() async {
        StubProtocol.handler = { _ in (413, "") }
        await #expect(throws: SharedSecretRepositoryError.tooLarge) {
            try await repository.save(payload: Data(), expiration: Date(), selfDestruct: true)
        }
    }

    @Test func fetchDecodesAClientEncryptedPayload() async throws {
        StubProtocol.handler = { _ in (200, #"{"secret":"-_8","self_destruct":true,"client_encrypted":true}"#) }

        let secret = try await repository.fetch(id: SecretId(Self.id)!)

        #expect(secret == SharedSecret(content: .clientEncrypted(Data([0xFB, 0xFF])), selfDestruct: true))
    }

    @Test func fetchOfAMissingSecretIsNotFound() async {
        StubProtocol.handler = { _ in (404, "") }
        await #expect(throws: SharedSecretRepositoryError.notFound) {
            try await repository.fetch(id: SecretId(Self.id)!)
        }
    }

    @Test func metadataIsNilOnAServerThatPredatesTheEndpoint() async throws {
        StubProtocol.handler = { _ in (404, "Not Found") }
        #expect(try await repository.metadata(id: SecretId(Self.id)!) == nil)
    }

    @Test func metadataNeverHitsTheConsumingRoute() async throws {
        StubProtocol.handler = { _ in (200, #"{"exists":true,"client_encrypted":true,"self_destruct":true}"#) }

        let metadata = try await repository.metadata(id: SecretId(Self.id)!)

        #expect(metadata == SecretMetadata(exists: true, clientEncrypted: true, selfDestruct: true))
        #expect(StubProtocol.lastRequest?.url?.path == "/secret/\(Self.id)/meta")
    }
}
