use crate::{
    client::WhisperClient,
    config::{WhisperConfig, DEFAULT_URL},
    error::CliError,
    ui,
};
use console::style;
use std::str::FromStr;
use tracing::debug;
use url::Url;

pub async fn run(target: &ShareTarget) -> Result<(), CliError> {
    let (base_url, id) = match target.clone() {
        ShareTarget::FullUrl { base_url, id } => (base_url, id),
        ShareTarget::RawId(id) => {
            let url = WhisperConfig::load()
                .map(|c| c.url)
                .unwrap_or_else(|_| Url::parse(DEFAULT_URL).expect("DEFAULT_URL is valid"));
            (url, id)
        }
    };
    debug!("Resolved base_url={}, id={}", base_url, id);

    let spinner = ui::spinner("Fetching secret...");

    debug!("Fetching from {}", base_url);
    let client = WhisperClient::new(&base_url);
    let result = client.get_ephemeral_secret(&id).await?;
    debug!("Fetch complete");

    spinner.finish_and_clear();

    match result {
        Some(secret) => {
            if secret.self_destruct {
                eprintln!(
                    "{} This secret has been deleted after retrieval.",
                    style("warn:").yellow().bold()
                );
                eprintln!();
            }
            println!("{}", secret.secret);
        }
        None => {
            return Err(CliError::SecretExpiredOrNotFound);
        }
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub enum ShareTarget {
    FullUrl { base_url: Url, id: String },
    RawId(String),
}

impl FromStr for ShareTarget {
    type Err = CliError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // Try parsing as a full URL first
        if let Ok(url) = Url::parse(input) {
            let id = url
                .query_pairs()
                .find(|(k, _)| k == "shared_secret_id")
                .map(|(_, v)| v.to_string())
                .ok_or_else(|| {
                    CliError::InvalidShareTarget(
                        "URL has no shared_secret_id query parameter".to_string(),
                    )
                })?;

            if id.is_empty() {
                return Err(CliError::InvalidShareTarget("empty id in URL".to_string()));
            }

            let mut base_url = url.clone();
            base_url.set_path("");
            base_url.set_query(None);

            Ok(ShareTarget::FullUrl { base_url, id })
        } else {
            // Validate as UUID
            uuid::Uuid::parse_str(input).map_err(|_| {
                CliError::InvalidShareTarget(format!(
                    "'{}' is not a valid Whisper share link or UUID",
                    input
                ))
            })?;
            Ok(ShareTarget::RawId(input.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_url() {
        let target = ShareTarget::from_str(
            "https://whisper.example.com/get_secret?shared_secret_id=550e8400-e29b-41d4-a716-446655440000",
        )
        .unwrap();
        match target {
            ShareTarget::FullUrl { base_url, id } => {
                assert_eq!(base_url.as_str(), "https://whisper.example.com/");
                assert_eq!(id, "550e8400-e29b-41d4-a716-446655440000");
            }
            _ => panic!("Expected FullUrl"),
        }
    }

    #[test]
    fn test_parse_different_host() {
        let target = ShareTarget::from_str(
            "https://my-server.io/get_secret?shared_secret_id=550e8400-e29b-41d4-a716-446655440000",
        )
        .unwrap();
        match target {
            ShareTarget::FullUrl { base_url, id } => {
                assert_eq!(base_url.as_str(), "https://my-server.io/");
                assert_eq!(id, "550e8400-e29b-41d4-a716-446655440000");
            }
            _ => panic!("Expected FullUrl"),
        }
    }

    #[test]
    fn test_parse_raw_uuid() {
        let target = ShareTarget::from_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        match target {
            ShareTarget::RawId(id) => {
                assert_eq!(id, "550e8400-e29b-41d4-a716-446655440000");
            }
            _ => panic!("Expected RawId"),
        }
    }

    #[test]
    fn test_invalid_input() {
        assert!(ShareTarget::from_str("not-a-uuid").is_err());
    }

    #[test]
    fn invalid_input_error_is_user_friendly() {
        let err = ShareTarget::from_str("not a url").unwrap_err();
        let msg = err.to_string();
        // Must NOT leak the uuid crate's internal diagnostic.
        assert!(
            !msg.contains("urn:uuid"),
            "should not leak uuid crate internals: {msg}"
        );
        assert!(
            !msg.contains("invalid character"),
            "should not leak uuid crate internals: {msg}"
        );
        // But should still clearly say what's wrong.
        assert!(
            msg.contains("not a url"),
            "should echo the bad input: {msg}"
        );
        assert!(
            msg.to_lowercase().contains("whisper") || msg.to_lowercase().contains("share"),
            "should mention what kind of value is expected: {msg}"
        );
    }
}
