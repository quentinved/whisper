import SwiftUI
import WhisperCore

/// "● Opened & deleted · today 14:02", shared by the "Link ready" screen and
/// the Sent list so both say the same thing the same way as the website.
struct DeliveryStatusLabel: View {
    let status: SentStore.RowStatus?
    /// A self-destruct secret that was opened is gone; a reusable one isn't.
    let selfDestruct: Bool

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            Circle()
                .fill(dotColor)
                .frame(width: 8, height: 8)
                .padding(.top, 6)
            (Text(title)
                .font(.system(size: 15, weight: .semibold))
                .foregroundStyle(Theme.textPrimary)
                + Text("  " + detail)
                .font(.system(size: 14))
                .foregroundStyle(Theme.textMuted))
                .fixedSize(horizontal: false, vertical: true)
        }
        .accessibilityElement(children: .ignore)
        .accessibilityLabel("\(title), \(detail)")
        .accessibilityIdentifier("delivery-status")
    }

    private var outcome: DeliveryStatus.Outcome? {
        guard case .known(let status) = status else { return nil }
        return status.outcome(selfDestruct: selfDestruct)
    }

    private var title: String {
        switch status {
        case .none, .checking: return "Checking"
        case .unavailable: return "Unknown"
        case .known:
            switch outcome {
            case .waiting: return "Waiting"
            case .openedAndDeleted: return "Opened & deleted"
            case .openedStillReadable, .openedThenExpired: return "Opened"
            case .expiredUnopened, .none: return "Expired"
            }
        }
    }

    private var detail: String {
        switch status {
        case .none, .checking: return "…"
        case .unavailable: return "couldn't check"
        case .known:
            switch outcome {
            case .waiting(let expiresAt):
                return "expires " + expiresAt.formatted(.relative(presentation: .named))
            case .openedAndDeleted(let at):
                return at.map(Self.moment) ?? ""
            case .openedStillReadable(let at, let until):
                return (at.map(Self.moment) ?? "") + " · readable until " + Self.moment(until)
            case .openedThenExpired(let at):
                return (at.map(Self.moment) ?? "") + " · expired"
            case .expiredUnopened, .none:
                return "never opened"
            }
        }
    }

    /// Mint once it was delivered, amber while it's still out there, grey once it's moot.
    private var dotColor: Color {
        switch outcome {
        case .openedAndDeleted, .openedStillReadable, .openedThenExpired: return Theme.mint
        case .waiting: return Theme.warning
        case .expiredUnopened, .none: return Theme.textMuted
        }
    }

    private static func moment(_ date: Date) -> String {
        Calendar.current.isDateInToday(date)
            ? "today " + date.formatted(date: .omitted, time: .shortened)
            : date.formatted(date: .abbreviated, time: .shortened)
    }
}
