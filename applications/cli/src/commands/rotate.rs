use crate::{env_whisper, error::CliError, session::Session, ui};
use console::style;
use tracing::debug;

pub async fn run(name: &str) -> Result<(), CliError> {
    crate::config::ensure_exists()?;

    let uuid = env_whisper::get(name)?.ok_or_else(|| CliError::SecretNotFound {
        name: name.to_string(),
    })?;

    if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        return Err(CliError::NotATerminal);
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // Sync test + block_on (see push.rs::tests for rationale).
    #[test]
    fn rotate_errors_with_not_a_terminal() {
        let _g = crate::config::CWD_LOCK.lock().unwrap();
        let dir = TempDir::new().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        fs::write(
            ".whisperrc",
            r#"{"passphrase":"test","url":"http://localhost"}"#,
        )
        .unwrap();
        fs::write(".env.whisper", "FOO=00000000-0000-0000-0000-000000000000\n").unwrap();
        let result = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(run("FOO"));

        std::env::set_current_dir(prev).unwrap();
        match result {
            Err(CliError::NotATerminal) => {}
            other => panic!("expected NotATerminal, got {other:?}"),
        }
    }
}
