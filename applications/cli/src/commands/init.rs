use crate::{
    client::WhisperClient,
    config::{CONFIG_FILE, DEFAULT_URL},
    error::CliError,
    ui,
};
use console::style;
use rand::Rng;
use std::io::Write;
use std::path::Path;
use tracing::debug;
use url::Url;

// Used for ephemeral secret expiration to share the passphrase on init. We set a long expiration (24h) to give enough time for every teammate to retrieve it. Self-destruct is disabled so multiple people can open the link.
const TWENTY_FOUR_HOURS: i64 = 24 * 60 * 60;

pub async fn run(url: Option<&str>, manual_passphrase: bool) -> Result<(), CliError> {
    let config_path = Path::new(CONFIG_FILE);
    if config_path.exists() {
        eprintln!(
            "{} .whisperrc already exists in this directory.",
            style("skip:").yellow().bold()
        );
        return Ok(());
    }

    let was_default = url.is_none();
    let url_str = url.unwrap_or(DEFAULT_URL);
    let parsed_url = Url::parse(url_str)
        .map_err(|e| CliError::InvalidShareTarget(format!("invalid URL '{}': {}", url_str, e)))?;

    print_server_notice(&mut std::io::stdout(), &parsed_url, was_default)
        .map_err(CliError::ConfigWrite)?;

    let passphrase = if manual_passphrase {
        dialoguer::Password::new()
            .with_prompt("Passphrase")
            .with_confirmation("Confirm passphrase", "Passphrases don't match")
            .interact()
            .map_err(|e| CliError::Input(e.to_string()))?
    } else {
        generate_passphrase()
    };

    let share_url = share_passphrase(&parsed_url, &passphrase).await?;
    write_config(config_path, url_str, &passphrase)?;
    let gitignore_modified = crate::config::append_to_gitignore(CONFIG_FILE)?;
    print_success(&mut std::io::stdout(), &share_url, gitignore_modified)
        .map_err(CliError::ConfigWrite)?;
    print_tips(&mut std::io::stdout()).map_err(CliError::ConfigWrite)?;
    crate::clipboard::prompt_and_copy(share_url.as_str())?;

    Ok(())
}

pub fn write_config(path: &Path, url: &str, passphrase: &str) -> Result<(), CliError> {
    let config = serde_json::json!({
        "url": url,
        "passphrase": passphrase,
    });

    debug!("Writing config to {}", CONFIG_FILE);
    std::fs::write(path, serde_json::to_string_pretty(&config).unwrap() + "\n")
        .map_err(CliError::ConfigWrite)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
            .map_err(CliError::ConfigWrite)?;
    }

    Ok(())
}

pub async fn share_passphrase(url: &Url, passphrase: &str) -> Result<Url, CliError> {
    let spinner = ui::spinner("Sharing passphrase via Whisper...");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let expiration = now + TWENTY_FOUR_HOURS;

    debug!("Creating ephemeral secret on {}", url);
    let client = WhisperClient::new(url);
    let share_url = client
        .create_ephemeral_secret(passphrase, expiration, false)
        .await?;
    debug!("Ephemeral secret created");

    spinner.finish_and_clear();
    Ok(share_url)
}

fn print_server_notice(out: &mut impl Write, url: &Url, was_default: bool) -> std::io::Result<()> {
    if was_default {
        writeln!(
            out,
            "  {} Using default server: {} {}",
            style("-->").dim(),
            style(url.as_str()).cyan(),
            style("(pass --url for self-hosted)").dim()
        )?;
    } else {
        writeln!(
            out,
            "  {} Using server: {}",
            style("-->").dim(),
            style(url.as_str()).cyan()
        )?;
    }
    Ok(())
}

fn print_success(
    out: &mut impl Write,
    share_url: &Url,
    gitignore_modified: bool,
) -> std::io::Result<()> {
    writeln!(
        out,
        "{} Created {}",
        style("done").green().bold(),
        style(".whisperrc").cyan()
    )?;
    writeln!(out)?;
    writeln!(
        out,
        "  Share this link with your team {}:",
        style("(expires in 24h)").dim()
    )?;
    writeln!(
        out,
        "  \x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
        share_url,
        style(share_url.as_str()).underlined()
    )?;
    writeln!(
        out,
        "  {} This link contains your team's passphrase — treat it as a secret.",
        style("!").yellow().bold()
    )?;
    writeln!(out)?;
    if gitignore_modified {
        writeln!(
            out,
            "  {} Added {} to {}.",
            style("-->").dim(),
            style(".whisperrc").cyan(),
            style(".gitignore").cyan()
        )?;
    } else {
        writeln!(
            out,
            "  {} {} already ignored in {}.",
            style("-->").dim(),
            style(".whisperrc").cyan(),
            style(".gitignore").cyan()
        )?;
    }
    Ok(())
}

fn print_tips(out: &mut impl Write) -> std::io::Result<()> {
    writeln!(out)?;
    writeln!(
        out,
        "  {} Tip: {} is a shortcut for {}.",
        style("-->").dim(),
        style("ws").cyan(),
        style("whisper-secrets").cyan()
    )?;
    writeln!(
        out,
        "  {} Tip: use {} / {} for .env workflow, {} for one-off secrets.",
        style("-->").dim(),
        style("ws import").cyan(),
        style("ws push").cyan(),
        style("ws share").cyan()
    )?;
    Ok(())
}

fn generate_passphrase() -> String {
    let bytes: [u8; 24] = rand::rng().random();
    base64_url::encode(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_server_notice_shows_default_url() {
        let url = Url::parse(DEFAULT_URL).unwrap();
        let mut buf = Vec::new();
        print_server_notice(&mut buf, &url, true).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("whisper.quentinvedrenne.com"), "got: {s}");
        assert!(s.contains("default"), "should mark default: {s}");
    }

    #[test]
    fn print_server_notice_shows_custom_url() {
        let url = Url::parse("https://my-host.example/").unwrap();
        let mut buf = Vec::new();
        print_server_notice(&mut buf, &url, false).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("my-host.example"), "got: {s}");
        assert!(!s.contains("default"), "should not mark default: {s}");
    }

    #[test]
    fn print_success_includes_passphrase_warning() {
        let url = Url::parse("https://example.com/get_secret?shared_secret_id=abc").unwrap();
        let mut buf = Vec::new();
        print_success(&mut buf, &url, true).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(
            s.to_lowercase().contains("passphrase"),
            "should warn link carries passphrase: {s}"
        );
    }

    #[test]
    fn print_tips_mentions_ws_alias_and_both_workflows() {
        let mut buf = Vec::new();
        print_tips(&mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains(" ws "), "should mention ws alias: {s}");
        assert!(
            s.contains("import") || s.contains("push"),
            "should mention managed workflow: {s}"
        );
        assert!(
            s.contains("share"),
            "should mention ephemeral workflow: {s}"
        );
    }
}
