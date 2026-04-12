use crate::{
    client::WhisperClient,
    commands::{get::ShareTarget, init::write_config, pull},
    config::CONFIG_FILE,
    env_whisper::ENV_WHISPER_FILE,
    error::CliError,
    ui,
};
use console::style;
use std::path::Path;
use tracing::debug;

pub async fn run(target: &ShareTarget) -> Result<(), CliError> {
    let config_path = Path::new(CONFIG_FILE);
    if config_path.exists() {
        eprintln!(
            "{} .whisperrc already exists in this directory.",
            style("skip:").yellow().bold()
        );
        return Ok(());
    }

    let (base_url, id) = match target.clone() {
        ShareTarget::FullUrl { base_url, id } => (base_url, id),
        ShareTarget::RawId(_) => {
            return Err(CliError::InvalidShareTarget(
                "join requires a full Whisper URL, not a raw UUID".to_string(),
            ));
        }
    };

    let spinner = ui::spinner("Fetching passphrase...");

    debug!("Fetching ephemeral secret from {}", base_url);
    let client = WhisperClient::new(&base_url);
    let result = client.get_ephemeral_secret(&id).await?;

    spinner.finish_and_clear();

    let passphrase = match result {
        Some(secret) => secret.secret,
        None => return Err(CliError::SecretExpiredOrNotFound),
    };

    write_config(config_path, base_url.as_str(), &passphrase)?;

    println!(
        "{} Joined project via Whisper link",
        style("done").green().bold()
    );

    if Path::new(ENV_WHISPER_FILE).exists() {
        println!(
            "{} .env.whisper detected, pulling secrets...",
            style("auto:").cyan().bold()
        );
        pull::run().await?;
    } else {
        eprintln!();
        eprintln!(
            "{} No .env.whisper found. Are you in the right directory?",
            style("warn:").yellow().bold()
        );
        eprintln!(
            "  Clone the project repo first, then run {} to get your secrets.",
            style("whisper-secrets pull").cyan()
        );
    }

    Ok(())
}
