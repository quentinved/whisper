import SwiftUI

/// "● Opened · today 14:02", shared by the "Link ready" screen and the Sent
/// list so both say the same thing the same way as the website.
struct DeliveryStatusLabel: View {
    let status: SentStore.RowStatus?

    var body: some View {
        HStack(spacing: 6) {
            Circle()
                .fill(dotColor)
                .frame(width: 8, height: 8)
            Text(title)
                .font(.system(size: 15, weight: .semibold))
                .foregroundStyle(Theme.textPrimary)
            Text(detail)
                .font(.system(size: 14))
                .foregroundStyle(Theme.textMuted)
        }
        .accessibilityElement(children: .ignore)
        .accessibilityLabel("\(title), \(detail)")
        .accessibilityIdentifier("delivery-status")
    }

    private var title: String {
        switch status {
        case .none, .checking: return "Checking"
        case .unavailable: return "Unknown"
        case .known(let s):
            switch s.state {
            case .waiting: return "Waiting"
            case .opened: return "Opened"
            case .expired: return "Expired"
            }
        }
    }

    private var detail: String {
        switch status {
        case .none, .checking: return "…"
        case .unavailable: return "couldn't check"
        case .known(let s):
            switch s.state {
            case .waiting:
                return "expires " + s.expiresAt.formatted(.relative(presentation: .named))
            case .opened:
                return s.openedAt.map(Self.moment) ?? ""
            case .expired:
                return "never opened"
            }
        }
    }

    private var dotColor: Color {
        guard case .known(let s) = status else { return Theme.textMuted }
        switch s.state {
        case .opened: return Theme.accent
        case .waiting: return Theme.warning
        case .expired: return Theme.textMuted
        }
    }

    private static func moment(_ date: Date) -> String {
        Calendar.current.isDateInToday(date)
            ? "today " + date.formatted(date: .omitted, time: .shortened)
            : date.formatted(date: .abbreviated, time: .shortened)
    }
}
