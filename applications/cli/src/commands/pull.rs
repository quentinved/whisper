use crate::{env_whisper, error::CliError, session::Session, ui};
use console::style;
use indicatif::ProgressBar;
use std::collections::BTreeMap;
use std::path::Path;
use tracing::debug;

pub async fn run() -> Result<(), CliError> {
    crate::config::ensure_exists()?;

    let entries = env_whisper::read()?;
    debug!("Found {} entries in .env.whisper", entries.len());

    if entries.is_empty() {
        println!(
            "{} No secrets in .env.whisper.",
            style("skip:").yellow().bold()
        );
        return Ok(());
    }

    if !confirm_overwrite(&entries)? {
        println!("{} Aborted.", style("skip:").yellow().bold());
        return Ok(());
    }

    let spinner = ui::spinner("Deriving key...");

    let session = Session::load()?;
    let env_lines = fetch_and_decrypt(&entries, &session, &spinner).await?;
    write_env_file(&env_lines)?;

    spinner.finish_and_clear();

    println!(
        "{} Pulled {} secrets to {}",
        style("done").green().bold(),
        style(entries.len()).cyan(),
        style(".env").cyan()
    );
    Ok(())
}

fn confirm_overwrite(tracked: &BTreeMap<String, String>) -> Result<bool, CliError> {
    let env_path = Path::new(".env");
    if !env_path.exists() {
        return Ok(true);
    }

    let local = env_whisper::read_env_file();
    let local_only: Vec<&String> = local.keys().filter(|k| !tracked.contains_key(*k)).collect();

    if local_only.is_empty() {
        return dialoguer::Confirm::new()
            .with_prompt("This will overwrite your .env file. Continue?")
            .default(true)
            .interact()
            .map_err(|e| CliError::Input(e.to_string()));
    }

    eprintln!(
        "{} Your .env has {} local-only entries not tracked by whisper-secrets:",
        style("warn:").yellow().bold(),
        local_only.len()
    );
    for name in &local_only {
        eprintln!("  - {}", style(name).yellow());
    }
    eprintln!("  These will be lost after pull.");
    eprintln!();

    dialoguer::Confirm::new()
        .with_prompt("Overwrite .env anyway?")
        .default(false)
        .interact()
        .map_err(|e| CliError::Input(e.to_string()))
}

async fn fetch_and_decrypt(
    entries: &BTreeMap<String, String>,
    session: &Session,
    spinner: &ProgressBar,
) -> Result<Vec<String>, CliError> {
    let mut env_lines = Vec::new();

    for (name, uuid) in entries {
        spinner.set_message(format!("Pulling {}...", name));

        debug!("Fetching {} (id={})", name, uuid);
        let payload_b64 =
            session
                .client()
                .get_secret(uuid)
                .await?
                .ok_or_else(|| CliError::NotFoundOnServer {
                    name: name.clone(),
                    id: uuid.clone(),
                })?;

        let payload = base64_url::decode(&payload_b64)
            .map_err(|_| CliError::Base64(format!("Invalid base64 from server for {}", name)))?;

        let decrypted =
            session
                .crypto()
                .decrypt(&payload)
                .map_err(|e| CliError::DecryptionFailed {
                    name: name.clone(),
                    reason: e.to_string(),
                })?;
        debug!("Decrypted {}", name);

        if decrypted.contains('\n') {
            spinner.finish_and_clear();
            return Err(CliError::MultilineValue { name: name.clone() });
        }

        env_lines.push(format!("{}={}", name, decrypted));
    }

    Ok(env_lines)
}

fn write_env_file(lines: &[String]) -> Result<(), CliError> {
    let content = lines.join("\n") + "\n";
    std::fs::write(".env", &content).map_err(CliError::EnvWrite)
}
