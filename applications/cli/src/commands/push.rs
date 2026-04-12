use crate::{env_whisper, error::CliError, session::Session, ui};
use console::style;
use tracing::debug;
use uuid::Uuid;

pub async fn run(name: Option<&str>) -> Result<(), CliError> {
    match name {
        Some(name) => push_single(name).await,
        None => push_interactive().await,
    }
}

async fn push_single(name: &str) -> Result<(), CliError> {
    if env_whisper::get(name)?.is_some() {
        return Err(CliError::SecretAlreadyExists {
            name: name.to_string(),
        });
    }

    let secret_value = dialoguer::Password::new()
        .with_prompt(format!("Value for {}", name))
        .interact()
        .map_err(|e| CliError::Input(e.to_string()))?;

    upload_secret(name, &secret_value).await
}

async fn push_interactive() -> Result<(), CliError> {
    let tracked = env_whisper::read()?;
    let local = env_whisper::read_env_file();

    let untracked: Vec<(String, String)> = local
        .into_iter()
        .filter(|(k, _)| !tracked.contains_key(k))
        .collect();

    if untracked.is_empty() {
        println!(
            "{} No untracked secrets found in .env",
            style("skip:").yellow().bold()
        );
        return Ok(());
    }

    let labels: Vec<String> = untracked.iter().map(|(k, _)| k.clone()).collect();

    let selections = dialoguer::MultiSelect::new()
        .with_prompt("Select secrets to push")
        .items(&labels)
        .interact()
        .map_err(|e| CliError::Input(e.to_string()))?;

    if selections.is_empty() {
        println!("{} Nothing selected", style("skip:").yellow().bold());
        return Ok(());
    }

    for idx in selections {
        let (name, value) = &untracked[idx];
        upload_secret(name, value).await?;
    }

    Ok(())
}

async fn upload_secret(name: &str, value: &str) -> Result<(), CliError> {
    let spinner = ui::spinner(&format!("Encrypting and uploading {}...", name));

    let session = Session::load()?;
    let payload = session.crypto().encrypt(value)?;
    let payload_b64 = base64_url::encode(&payload);
    debug!("Encryption done, payload size: {} bytes", payload_b64.len());

    let id = Uuid::new_v4().to_string();
    debug!("Uploading secret id={}", id);
    session.client().put_secret(&id, &payload_b64).await?;
    debug!("Upload complete");

    env_whisper::set(name, &id)?;

    spinner.finish_and_clear();

    println!(
        "{} Stored {}",
        style("done").green().bold(),
        style(name).cyan()
    );
    Ok(())
}
