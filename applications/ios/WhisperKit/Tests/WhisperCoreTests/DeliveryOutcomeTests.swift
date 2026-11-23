import Foundation
import Testing
import WhisperCore

@Suite struct DeliveryOutcomeTests {
    static let now = Date(timeIntervalSince1970: 1_800_000_000)
    static let opened = now.addingTimeInterval(-600)
    static let later = now.addingTimeInterval(3600)
    static let earlier = now.addingTimeInterval(-60)

    @Test func waitingStaysWaitingWhateverTheKind() {
        let status = DeliveryStatus(state: .waiting, expiresAt: Self.later, openedAt: nil)
        #expect(status.outcome(selfDestruct: true, now: Self.now) == .waiting(expiresAt: Self.later))
        #expect(status.outcome(selfDestruct: false, now: Self.now) == .waiting(expiresAt: Self.later))
    }

    @Test func anOpenedSelfDestructSecretIsDeleted() {
        let status = DeliveryStatus(state: .opened, expiresAt: Self.later, openedAt: Self.opened)
        #expect(status.outcome(selfDestruct: true, now: Self.now) == .openedAndDeleted(openedAt: Self.opened))
    }

    @Test func anOpenedReusableSecretIsReadableUntilItExpires() {
        let live = DeliveryStatus(state: .opened, expiresAt: Self.later, openedAt: Self.opened)
        #expect(
            live.outcome(selfDestruct: false, now: Self.now)
                == .openedStillReadable(openedAt: Self.opened, until: Self.later))

        let gone = DeliveryStatus(state: .opened, expiresAt: Self.earlier, openedAt: Self.opened)
        #expect(gone.outcome(selfDestruct: false, now: Self.now) == .openedThenExpired(openedAt: Self.opened))
    }

    @Test func neverOpenedAndExpired() {
        let status = DeliveryStatus(state: .expired, expiresAt: Self.earlier, openedAt: nil)
        #expect(status.outcome(selfDestruct: true, now: Self.now) == .expiredUnopened)
    }
}
