use crate::{env_whisper, error::CliError, session::Session, ui};
use console::style;
use tracing::debug;

pub async fn run(name: &str) -> Result<(), CliError> {
    let uuid = env_whisper::get(name)?.ok_or_else(|| CliError::SecretNotFound {
        name: name.to_string(),
    })?;

    let new_value = dialoguer::Password::new()
        .with_prompt(format!("New value for {}", name))
        .interact()
        .map_err(|e| CliError::Input(e.to_string()))?;

    let spinner = ui::spinner("Encrypting and uploading...");

    let session = Session::load()?;
    debug!("Encrypting new value for {}", name);
    let payload = session.crypto().encrypt(&new_value)?;
    let payload_b64 = base64_url::encode(&payload);

    debug!("Uploading to id={}", uuid);
    session.client().put_secret(&uuid, &payload_b64).await?;
    debug!("Upload complete");

    spinner.finish_and_clear();

    println!(
        "{} Rotated {}",
        style("done").green().bold(),
        style(name).cyan()
    );
    Ok(())
}
