use crate::{env_whisper, error::CliError, session::Session, ui};
use console::style;
use tracing::debug;
use uuid::Uuid;

pub async fn run(name: &str) -> Result<(), CliError> {
    if env_whisper::get(name)?.is_some() {
        return Err(CliError::SecretAlreadyExists {
            name: name.to_string(),
        });
    }

    let secret_value = dialoguer::Password::new()
        .with_prompt(format!("Value for {}", name))
        .interact()
        .map_err(|e| CliError::Input(e.to_string()))?;

    let spinner = ui::spinner("Encrypting and uploading...");

    let session = Session::load()?;
    let payload = session.crypto().encrypt(&secret_value)?;
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
