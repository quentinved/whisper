use crate::{
    client::WhisperClient,
    config::{CONFIG_FILE, DEFAULT_URL},
    error::CliError,
    ui,
};
use console::style;
use rand::Rng;
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

    let url_str = url.unwrap_or(DEFAULT_URL);
    let url = Url::parse(url_str)
        .map_err(|e| CliError::InvalidShareTarget(format!("invalid URL '{}': {}", url_str, e)))?;

    let passphrase = if manual_passphrase {
        dialoguer::Password::new()
            .with_prompt("Passphrase")
            .with_confirmation("Confirm passphrase", "Passphrases don't match")
            .interact()
            .map_err(|e| CliError::Input(e.to_string()))?
    } else {
        generate_passphrase()
    };

    write_config(config_path, url_str, &passphrase)?;
    let share_url = share_passphrase(&url, &passphrase).await?;
    print_success(&share_url);

    Ok(())
}

fn write_config(path: &Path, url: &str, passphrase: &str) -> Result<(), CliError> {
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

async fn share_passphrase(url: &Url, passphrase: &str) -> Result<Url, CliError> {
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

fn print_success(share_url: &Url) {
    println!(
        "{} Created {}",
        style("done").green().bold(),
        style(".whisperrc").cyan()
    );
    println!();
    println!(
        "  Share this link with your team {}:",
        style("(expires in 24h)").dim()
    );
    println!(
        "  \x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\",
        share_url,
        style(share_url.as_str()).underlined()
    );
    println!();
    println!(
        "  {} Add {} to your {}.",
        style("-->").dim(),
        style(".whisperrc").cyan(),
        style(".gitignore").cyan()
    );
}

fn generate_passphrase() -> String {
    let bytes: [u8; 24] = rand::rng().random();
    base64_url::encode(&bytes)
}
