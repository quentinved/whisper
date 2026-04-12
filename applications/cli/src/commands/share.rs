use crate::{
    client::WhisperClient,
    config::{WhisperConfig, DEFAULT_URL},
    error::CliError,
    ui,
};
use console::style;
use tracing::debug;
use url::Url;

pub async fn run(expiration: &str, no_self_destruct: bool) -> Result<(), CliError> {
    let base_url = WhisperConfig::load()
        .map(|c| c.url)
        .unwrap_or_else(|_| Url::parse(DEFAULT_URL).expect("DEFAULT_URL is valid"));
    debug!(
        "Server: {}, expiration: {}, self_destruct: {}",
        base_url, expiration, !no_self_destruct
    );

    let secret = dialoguer::Password::new()
        .with_prompt("Secret")
        .interact()
        .map_err(|e| CliError::Input(e.to_string()))?;

    let seconds = parse_duration(expiration)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let expiration_ts = now + seconds;

    let spinner = ui::spinner("Creating secret...");

    debug!("Creating ephemeral secret on {}", base_url);
    let client = WhisperClient::new(&base_url);
    let share_url = client
        .create_ephemeral_secret(&secret, expiration_ts, !no_self_destruct)
        .await?;
    debug!("Secret created");

    spinner.finish_and_clear();

    println!(
        "{} Secret created {}",
        style("done").green().bold(),
        if no_self_destruct {
            format!("(expires in {})", expiration)
        } else {
            format!(
                "(expires in {}, self-destructs after first view)",
                expiration
            )
        }
    );
    println!();
    println!(
        "  \x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
        share_url,
        style(share_url.as_str()).underlined()
    );

    crate::clipboard::prompt_and_copy(share_url.as_str())?;

    Ok(())
}

fn parse_duration(s: &str) -> Result<i64, CliError> {
    let s = s.trim().to_lowercase();
    let (num, multiplier) = if let Some(n) = s.strip_suffix('m') {
        (n, 60)
    } else if let Some(n) = s.strip_suffix('h') {
        (n, 3600)
    } else if let Some(n) = s.strip_suffix('d') {
        (n, 86400)
    } else {
        return Err(CliError::InvalidDuration(s));
    };

    let value: i64 = num
        .parse()
        .map_err(|_| CliError::InvalidDuration(s.clone()))?;

    if value <= 0 {
        return Err(CliError::InvalidDuration(s));
    }

    let total_seconds = value * multiplier;
    const MAX_EXPIRATION_SECONDS: i64 = 7 * 86400;
    if total_seconds > MAX_EXPIRATION_SECONDS {
        return Err(CliError::InvalidDuration(
            "duration cannot exceed 7 days".to_string(),
        ));
    }

    Ok(total_seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30m").unwrap(), 1800);
        assert_eq!(parse_duration("1h").unwrap(), 3600);
        assert_eq!(parse_duration("24h").unwrap(), 86400);
        assert_eq!(parse_duration("7d").unwrap(), 604800);
    }

    #[test]
    fn test_parse_duration_invalid() {
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("0h").is_err());
        assert!(parse_duration("-1d").is_err());
    }
}
