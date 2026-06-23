//! Per-page SEO metadata, built from the server's configured base URL so that
//! self-hosted instances emit canonical/Open Graph tags pointing at their own
//! domain (never a hardcoded one).

/// Default homepage `<title>` — the SEO surface (the visible slogan stays
/// "Built to Disappear." in the header).
pub const HOME_TITLE: &str = "Whisper — Zero-Knowledge Secret Sharing";

/// Default homepage meta/OG description.
pub const HOME_DESCRIPTION: &str = "Share passwords, API keys, and notes with zero-knowledge, end-to-end encrypted, self-destructing links. Encrypted in your browser — the server never sees your secret.";

/// SEO metadata for one rendered page. All URL fields are absolute.
#[derive(Debug, Clone)]
pub struct SeoMeta {
    pub title: String,
    pub description: String,
    /// Site root (trimmed of a trailing slash), e.g. `https://whisper.example.com`.
    pub base_url: String,
    /// Absolute canonical URL for this page (`base_url` + path).
    pub canonical: String,
    /// Absolute Open Graph image URL.
    pub og_image: String,
    /// `Some("noindex, nofollow")` for private pages; `None` for indexable ones.
    pub robots: Option<String>,
    /// Pre-serialized WebSite JSON-LD (valid JSON, safe to emit with `|safe`).
    pub json_ld_website: String,
    /// Pre-serialized SoftwareApplication JSON-LD (valid JSON, safe to emit with `|safe`).
    pub json_ld_software: String,
}

impl SeoMeta {
    fn build(
        base_url: &str,
        path: &str,
        title: &str,
        description: &str,
        robots: Option<String>,
    ) -> Self {
        let base = base_url.trim_end_matches('/').to_string();
        let canonical = format!("{base}{path}");
        let og_image = format!("{base}/assets/og-banner.png");
        let json_ld_website = serde_json::json!({
            "@context": "https://schema.org",
            "@type": "WebSite",
            "name": "Whisper",
            "description": description,
            "url": base,
        })
        .to_string();
        let json_ld_software = serde_json::json!({
            "@context": "https://schema.org",
            "@type": "SoftwareApplication",
            "name": "Whisper",
            "applicationCategory": "SecurityApplication",
            "operatingSystem": "Web, macOS, Windows, Linux",
            "description": description,
            "offers": { "@type": "Offer", "price": "0", "priceCurrency": "USD" },
            "url": base,
        })
        .to_string();
        Self {
            title: title.to_string(),
            description: description.to_string(),
            base_url: base,
            canonical,
            og_image,
            robots,
            json_ld_website,
            json_ld_software,
        }
    }

    /// Indexable public page.
    pub fn new(base_url: &str, path: &str, title: &str, description: &str) -> Self {
        Self::build(base_url, path, title, description, None)
    }

    /// Private page — emits `robots: noindex, nofollow`.
    pub fn private(base_url: &str, path: &str, title: &str, description: &str) -> Self {
        Self::build(
            base_url,
            path,
            title,
            description,
            Some("noindex, nofollow".to_string()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_absolute_canonical_and_image() {
        let s = SeoMeta::new("https://whisper.example.com", "/integrations", "T", "D");
        assert_eq!(s.canonical, "https://whisper.example.com/integrations");
        assert_eq!(
            s.og_image,
            "https://whisper.example.com/assets/og-banner.png"
        );
        assert_eq!(s.base_url, "https://whisper.example.com");
        assert!(s.robots.is_none());
    }

    #[test]
    fn trims_trailing_slash_so_no_double_slash() {
        let s = SeoMeta::new("https://whisper.example.com/", "/", "T", "D");
        assert_eq!(s.canonical, "https://whisper.example.com/");
        assert_eq!(
            s.og_image,
            "https://whisper.example.com/assets/og-banner.png"
        );
    }

    #[test]
    fn private_sets_noindex() {
        let s = SeoMeta::private("https://x.io", "/get_secret", "T", "D");
        assert_eq!(s.robots.as_deref(), Some("noindex, nofollow"));
    }

    #[test]
    fn json_ld_is_valid_json_even_with_apostrophe() {
        // Descriptions like "Whisper's privacy policy" must not break JSON-LD.
        let s = SeoMeta::new("https://x.io", "/privacy", "T", "Whisper's policy & more");
        let website: serde_json::Value = serde_json::from_str(&s.json_ld_website).unwrap();
        assert_eq!(website["@type"], "WebSite");
        assert_eq!(website["description"], "Whisper's policy & more");
        let app: serde_json::Value = serde_json::from_str(&s.json_ld_software).unwrap();
        assert_eq!(app["@type"], "SoftwareApplication");
        assert_eq!(app["url"], "https://x.io");
    }
}
