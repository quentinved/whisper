use crate::{env_whisper, error::CliError, session::Session, ui};
use console::style;
use std::path::Path;
use tracing::debug;

pub async fn run(name: &str) -> Result<(), CliError> {
    crate::config::ensure_exists()?;

    let uuid = env_whisper::get(name)?.ok_or_else(|| CliError::SecretNotFound {
        name: name.to_string(),
    })?;

    let spinner = ui::spinner("Deleting...");

    let session = Session::load()?;
    debug!("Deleting {} (id={})", name, uuid);
    session.client().delete_secret(&uuid).await?;
    debug!("Deleted from server");

    env_whisper::remove(name)?;
    debug!("Removed from .env.whisper");

    // Also remove from .env if exists
    if Path::new(".env").exists() {
        let content = std::fs::read_to_string(".env").map_err(CliError::EnvRead)?;
        let filtered: String = content
            .lines()
            .filter(|line| !line.starts_with(&format!("{}=", name)))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        std::fs::write(".env", filtered).map_err(CliError::EnvWrite)?;
        debug!("Cleaned {} from .env", name);
    }

    spinner.finish_and_clear();

    println!(
        "{} Removed {}",
        style("done").green().bold(),
        style(name).cyan()
    );
    Ok(())
}
