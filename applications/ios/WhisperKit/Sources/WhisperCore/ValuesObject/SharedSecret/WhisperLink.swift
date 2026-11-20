import Foundation

/// A share link: the secret's id plus the key that decrypts it.
///
/// The key lives in the URL fragment, which is the whole point — browsers never
/// transmit a fragment, so the server learns the id but never the key. The app
/// upholds the same rule: `id` goes over the network, `key` never does.
public struct WhisperLink: Equatable, Sendable {
    public let id: SecretId
    public let key: SecretKey
    public let server: URL

    public init(id: SecretId, key: SecretKey, server: URL) {
        self.id = id
        self.key = key
        self.server = server
    }

    /// Accepts anything a person might realistically paste: a link from the web
    /// UI (`/?shared_secret_id=…`), from the CLI or Slack (`/get_secret?…`), or
    /// the app's own `whisper://` deep link. Surrounding whitespace and
    /// Markdown-style angle brackets are tolerated because messaging apps add
    /// them.
    ///
    /// The link's own origin wins, so a link from a self-hosted instance is
    /// fetched from that instance. `whisper://` links carry no host and use
    /// `fallbackServer`; without one they are rejected.
    public init?(parsing raw: String, fallbackServer: URL? = nil) {
        let trimmed = raw.trimmingCharacters(in: .whitespacesAndNewlines)
            .trimmingCharacters(in: CharacterSet(charactersIn: "<>"))

        guard let components = URLComponents(string: trimmed),
            let rawId = components.queryItems?
                .first(where: { $0.name == "shared_secret_id" })?.value,
            let id = SecretId(rawId),
            // The fragment is `k=<base64url>`; reject anything else rather than
            // silently producing a link that cannot decrypt.
            let fragment = components.fragment,
            fragment.hasPrefix("k="),
            let key = SecretKey(base64URL: String(fragment.dropFirst(2))),
            let server = Self.origin(of: components) ?? Self.deepLink(components, fallbackServer)
        else { return nil }

        self.init(id: id, key: key, server: server)
    }

    /// Builds the canonical form, matching `ephemeral_share_url` in
    /// `applications/cli/src/client.rs` so links look identical whichever
    /// client made them.
    public var url: URL {
        var components = URLComponents(
            url: server.appendingPathComponent("get_secret"),
            resolvingAgainstBaseURL: false
        )!
        components.queryItems = [URLQueryItem(name: "shared_secret_id", value: id.rawValue)]
        components.fragment = "k=\(key.base64URL)"
        return components.url!
    }

    private static func origin(of components: URLComponents) -> URL? {
        guard let scheme = components.scheme?.lowercased(),
            scheme == "https" || scheme == "http",
            let host = components.host
        else { return nil }

        var base = URLComponents()
        base.scheme = scheme
        base.host = host
        base.port = components.port
        return base.url
    }

    private static func deepLink(_ components: URLComponents, _ fallback: URL?) -> URL? {
        components.scheme?.lowercased() == "whisper" ? fallback : nil
    }
}
