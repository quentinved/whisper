use crate::{env_whisper, error::CliError, session::Session, ui};
use console::style;
use indicatif::ProgressBar;
use std::path::Path;
use tracing::debug;
use uuid::Uuid;

pub async fn run() -> Result<(), CliError> {
    let entries = parse_env_file()?;
    if entries.is_empty() {
        println!(
            "{} No entries found in .env.",
            style("skip:").yellow().bold()
        );
        return Ok(());
    }

    let spinner = ui::spinner("Deriving key...");

    let session = Session::load()?;
    let (imported, skipped) = upload_entries(&entries, &session, &spinner).await?;

    spinner.finish_and_clear();
    print_summary(imported, skipped);

    Ok(())
}

fn parse_env_file() -> Result<Vec<(String, String)>, CliError> {
    let env_path = Path::new(".env");
    if !env_path.exists() {
        return Err(CliError::NoEnvFile);
    }

    let content = std::fs::read_to_string(env_path).map_err(CliError::EnvWrite)?;

    let entries: Vec<(String, String)> = content
        .lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .map(|(k, v)| (k.trim().to_string(), v.to_string()))
        .collect();

    debug!("Found {} entries in .env", entries.len());
    Ok(entries)
}

async fn upload_entries(
    entries: &[(String, String)],
    session: &Session,
    spinner: &ProgressBar,
) -> Result<(usize, usize), CliError> {
    let mut imported = 0;
    let mut skipped = 0;

    for (name, value) in entries {
        if env_whisper::get(name)?.is_some() {
            debug!("Skipping {} (already tracked)", name);
            skipped += 1;
            continue;
        }

        spinner.set_message(format!("Uploading {}...", name));

        debug!("Encrypting {}", name);
        let payload = session.crypto().encrypt(value)?;
        let payload_b64 = base64_url::encode(&payload);

        let id = Uuid::new_v4().to_string();
        debug!("Uploading {} (id={})", name, id);
        session.client().put_secret(&id, &payload_b64).await?;
        env_whisper::set(name, &id)?;
        debug!("Stored {}", name);

        imported += 1;
    }

    Ok((imported, skipped))
}

fn print_summary(imported: usize, skipped: usize) {
    println!(
        "{} Imported {} secrets.",
        style("done").green().bold(),
        style(imported).cyan()
    );
    if skipped > 0 {
        println!(
            "     {} {} already in .env.whisper, skipped.",
            style(skipped).dim(),
            if skipped == 1 { "entry" } else { "entries" }
        );
    }
}
