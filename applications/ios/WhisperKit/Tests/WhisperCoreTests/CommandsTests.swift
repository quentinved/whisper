import Foundation
import Testing
import WhisperCore

@Suite struct CreateClientEncryptedSecretTests {
    @Test func rejectsBlankSecrets() {
        #expect(throws: CreateClientEncryptedSecretError.empty) {
            try CreateClientEncryptedSecret(plaintext: " \n ", expiration: .oneHour, selfDestruct: true)
        }
    }

    @Test func rejectsSecretsOverTheServerLimitInBytes() {
        // 16 385 four-byte emoji: well under 64 K characters, over 64 KB.
        let secret = String(repeating: "🔐", count: CreateClientEncryptedSecret.maxBytes / 4 + 1)
        #expect(throws: CreateClientEncryptedSecretError.tooLarge(bytes: secret.utf8.count)) {
            try CreateClientEncryptedSecret(plaintext: secret, expiration: .oneHour, selfDestruct: true)
        }
    }

    @Test(arguments: [-60.0, SecretExpiration.maximum + 60])
    func refusesACustomExpiryInThePastOrBeyondAWeek(offset: TimeInterval) async throws {
        let now = Date()
        let repository = InMemoryRepository()
        let command = try CreateClientEncryptedSecret(
            plaintext: "x", expiration: .custom(now.addingTimeInterval(offset)), selfDestruct: true)

        await #expect(throws: CreateClientEncryptedSecretError.invalidExpiration) {
            try await command.handle(encryption: ReversingEncryption(), repository: repository, now: now)
        }
        #expect(await repository.stored.isEmpty, "nothing may be uploaded for a refused expiry")
    }

    @Test func acceptsACustomExpiryWithinTheWeek() async throws {
        let now = Date()
        let repository = InMemoryRepository()
        let link = try await CreateClientEncryptedSecret(
            plaintext: "x", expiration: .custom(now.addingTimeInterval(3 * 24 * 3600)), selfDestruct: true
        ).handle(encryption: ReversingEncryption(), repository: repository, now: now).link

        #expect(await repository.stored[link.id] != nil)
    }

    @Test func storesOnlyCiphertextAndLinksToTheRepositoryServer() async throws {
        let repository = InMemoryRepository()
        let link = try await CreateClientEncryptedSecret(
            plaintext: "hunter2", expiration: .oneHour, selfDestruct: false
        ).handle(encryption: ReversingEncryption(), repository: repository).link

        #expect(link.server == repository.server)
        #expect(link.key == ReversingEncryption.key)
        let stored = await repository.stored[link.id]
        #expect(stored == SharedSecret(content: .clientEncrypted(Data("2retnuh".utf8)), selfDestruct: false))
    }
}

@Suite struct GetSecretByIdTests {
    @Test func decryptsAClientEncryptedSecretWithTheLinkKey() async throws {
        let repository = InMemoryRepository()
        let link = try await CreateClientEncryptedSecret(
            plaintext: "multi\nline ✅", expiration: .oneDay, selfDestruct: false
        ).handle(encryption: ReversingEncryption(), repository: repository).link

        let revealed = try await GetSecretById(link: link)
            .handle(encryption: ReversingEncryption(), repository: repository)

        #expect(revealed.text == "multi\nline ✅" && !revealed.burned)
    }

    @Test func returnsLegacyServerDecryptedSecretsAsIs() async throws {
        let repository = InMemoryRepository()
        let id = await repository.insert(SharedSecret(content: .plain("legacy"), selfDestruct: false))
        let link = WhisperLink(id: id, key: ReversingEncryption.key, server: repository.server)

        let revealed = try await GetSecretById(link: link)
            .handle(encryption: ReversingEncryption(), repository: repository)

        #expect(revealed.text == "legacy")
    }

    @Test func aSelfDestructSecretOpensExactlyOnce() async throws {
        let repository = InMemoryRepository()
        let link = try await CreateClientEncryptedSecret(
            plaintext: "once", expiration: .fiveMinutes, selfDestruct: true
        ).handle(encryption: ReversingEncryption(), repository: repository).link
        let command = GetSecretById(link: link)

        let first = try await command.handle(encryption: ReversingEncryption(), repository: repository)
        #expect(first.text == "once" && first.burned)

        await #expect(throws: SharedSecretRepositoryError.notFound) {
            try await command.handle(encryption: ReversingEncryption(), repository: repository)
        }
    }
}

@Suite struct GetSecretMetadataTests {
    @Test func doesNotConsumeASelfDestructSecret() async throws {
        let repository = InMemoryRepository()
        let id = await repository.insert(SharedSecret(content: .clientEncrypted(Data()), selfDestruct: true))

        let metadata = try await GetSecretMetadata(id: id).handle(repository: repository)

        #expect(metadata == SecretMetadata(exists: true, clientEncrypted: true, selfDestruct: true))
        #expect(await repository.stored[id] != nil)
    }

    @Test func aMissingSecretIsNotFound() async {
        let repository = InMemoryRepository()
        await #expect(throws: SharedSecretRepositoryError.notFound) {
            try await GetSecretMetadata(id: SecretId(UUID().uuidString)!).handle(repository: repository)
        }
    }

    @Test func anUnreachableServerMeansUnknownNotSafe() async throws {
        let repository = InMemoryRepository()
        await repository.failMetadata(with: .offline)

        let metadata = try await GetSecretMetadata(id: SecretId(UUID().uuidString)!)
            .handle(repository: repository)

        #expect(metadata == nil)
    }
}

@Suite struct GetDeliveryStatusTests {
    @Test func aNewSecretWaitsUntilItsFirstReveal() async throws {
        let repository = InMemoryRepository()
        let created = try await CreateClientEncryptedSecret(
            plaintext: "tracked", expiration: .oneHour, selfDestruct: true
        ).handle(encryption: ReversingEncryption(), repository: repository)
        let token = try #require(created.statusToken, "a tracking server must hand back a status token")

        let before = try await GetDeliveryStatus(token: token).handle(repository: repository)
        #expect(before?.state == .waiting && before?.openedAt == nil)

        _ = try await GetSecretById(link: created.link)
            .handle(encryption: ReversingEncryption(), repository: repository)

        let after = try await GetDeliveryStatus(token: token).handle(repository: repository)
        #expect(after?.state == .opened && after?.openedAt != nil)
    }

    @Test func anOlderServerLeavesTheSecretUntracked() async throws {
        let repository = InMemoryRepository()
        await repository.stopTrackingDelivery()

        let created = try await CreateClientEncryptedSecret(
            plaintext: "untracked", expiration: .oneHour, selfDestruct: false
        ).handle(encryption: ReversingEncryption(), repository: repository)

        #expect(created.statusToken == nil)
    }

    @Test func anUnknownTokenHasNoStatus() async throws {
        let token = StatusToken(UUID().uuidString.lowercased())!
        #expect(try await GetDeliveryStatus(token: token).handle(repository: InMemoryRepository()) == nil)
    }
}

@Suite struct StatusTokenTests {
    @Test(arguments: ["", "not-a-uuid", "../secret/x"])
    func rejectsAnythingButAUUID(raw: String) {
        #expect(StatusToken(raw) == nil)
    }

    @Test func decodingValidatesLikeInit() throws {
        let raw = "3f2b8c1e-9d4a-4b7e-a1c2-5e6f7a8b9c0d"
        let token = try JSONDecoder().decode(StatusToken.self, from: Data(#""\#(raw)""#.utf8))
        #expect(token.rawValue == raw)
        #expect(throws: DecodingError.self) {
            try JSONDecoder().decode(StatusToken.self, from: Data(#""../secret""#.utf8))
        }
    }
}
