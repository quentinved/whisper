import Foundation

/// A share link: the secret's id plus the key that decrypts it.
///
/// The key lives in the URL fragment, which is the whole point — browsers never
/// transmit a fragment, so the server learns the id but never the key. The app
/// upholds the same rule: `id` goes over the network, `key` never does.
struct WhisperLink: Equatable {
    let id: String
    let key: String
    let server: URL

    /// Builds the canonical form, matching `ephemeral_share_url` in
    /// `applications/cli/src/client.rs` so links look identical whichever
    /// client made them.
    var url: URL {
        var components = URLComponents(
            url: server.appendingPathComponent("get_secret"),
            resolvingAgainstBaseURL: false
        )!
        components.queryItems = [URLQueryItem(name: "shared_secret_id", value: id)]
        components.fragment = "k=\(key)"
        return components.url!
    }

    /// Accepts anything a person might realistically paste: a link from the web
    /// UI (`/?shared_secret_id=…`), from the CLI or Slack (`/get_secret?…`), or
    /// the app's own `whisper://` deep link. Surrounding whitespace and
    /// Markdown-style angle brackets are tolerated because messaging apps add
    /// them.
    static func parse(_ raw: String) -> WhisperLink? {
        let trimmed = raw.trimmingCharacters(in: .whitespacesAndNewlines)
            .trimmingCharacters(in: CharacterSet(charactersIn: "<>"))

        guard let components = URLComponents(string: trimmed) else { return nil }

        guard
            let id = components.queryItems?
                .first(where: { $0.name == "shared_secret_id" })?.value,
            UUID(uuidString: id) != nil
        else { return nil }

        // The fragment is `k=<base64url>`; reject anything else rather than
        // silently producing a link that cannot decrypt.
        guard let fragment = components.fragment,
            fragment.hasPrefix("k="),
            case let key = String(fragment.dropFirst(2)),
            !key.isEmpty,
            Base64URL.decode(key)?.count == 32
        else { return nil }

        guard let server = serverBase(of: components) else { return nil }

        return WhisperLink(id: id, key: key, server: server)
    }

    /// Recovers the origin the link points at, so a link from a self-hosted
    /// instance is fetched from that instance rather than the configured
    /// default. `whisper://` deep links carry no host, so they fall back to the
    /// user's configured server.
    private static func serverBase(of components: URLComponents) -> URL? {
        guard let scheme = components.scheme?.lowercased() else { return nil }

        if scheme == "whisper" { return nil }
        guard scheme == "https" || scheme == "http", let host = components.host else {
            return nil
        }

        var base = URLComponents()
        base.scheme = scheme
        base.host = host
        base.port = components.port
        return base.url
    }

    /// Parses a link, substituting `fallbackServer` when the source carried no
    /// origin of its own (a `whisper://` deep link).
    static func parse(_ raw: String, fallbackServer: URL) -> WhisperLink? {
        if let link = parse(raw) { return link }

        let trimmed = raw.trimmingCharacters(in: .whitespacesAndNewlines)
        guard let components = URLComponents(string: trimmed),
            components.scheme?.lowercased() == "whisper",
            let id = components.queryItems?
                .first(where: { $0.name == "shared_secret_id" })?.value,
            UUID(uuidString: id) != nil,
            let fragment = components.fragment, fragment.hasPrefix("k="),
            case let key = String(fragment.dropFirst(2)),
            Base64URL.decode(key)?.count == 32
        else { return nil }

        return WhisperLink(id: id, key: key, server: fallbackServer)
    }
}
