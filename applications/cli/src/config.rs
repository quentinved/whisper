use crate::error::CliError;
use serde::Deserialize;
use std::path::Path;
use url::Url;

pub const DEFAULT_URL: &str = "https://whisper.quentinvedrenne.com";
pub const CONFIG_FILE: &str = ".whisperrc";

/// Return `Err(CliError::MissingConfig)` if `.whisperrc` is absent from the current directory.
/// Call at the top of any command that requires a project to be initialized.
pub fn ensure_exists() -> Result<(), CliError> {
    if !Path::new(CONFIG_FILE).exists() {
        return Err(CliError::MissingConfig(CONFIG_FILE.to_string()));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct WhisperConfig {
    pub url: Url,
    pub passphrase: String,
}

impl WhisperConfig {
    pub fn load() -> Result<Self, CliError> {
        let config_path = Path::new(CONFIG_FILE);
        if !config_path.exists() {
            return Err(CliError::MissingConfig(CONFIG_FILE.to_string()));
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = config_path
                .metadata()
                .map_err(CliError::ConfigRead)?
                .permissions()
                .mode()
                & 0o777;
            if mode & 0o077 != 0 {
                eprintln!(
                    "\x1b[33mWARNING: {} has permissions {:o}, should be 600.\x1b[0m",
                    CONFIG_FILE, mode
                );
                eprintln!("         Run: chmod 600 {}", CONFIG_FILE);
            }
        }

        let content = std::fs::read_to_string(config_path).map_err(CliError::ConfigRead)?;

        let config: WhisperConfigFile =
            serde_json::from_str(&content).map_err(CliError::ConfigParse)?;

        let url = match config.url {
            Some(u) => Url::parse(&u)?,
            None => Url::parse(DEFAULT_URL).expect("DEFAULT_URL is a valid URL"),
        };

        let passphrase = config.passphrase.ok_or(CliError::MissingPassphrase)?;

        if passphrase.is_empty() {
            eprintln!(
                "\x1b[33mWARNING: passphrase in {} is empty. This is insecure.\x1b[0m",
                CONFIG_FILE
            );
        }

        Ok(Self { url, passphrase })
    }
}

#[derive(Debug, Deserialize)]
struct WhisperConfigFile {
    url: Option<String>,
    passphrase: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // ensure_exists uses the current working directory; lock it process-wide
    // because lib unit tests don't run with --test-threads=1.
    static CWD_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn ensure_exists_ok_when_config_present() {
        let _guard = CWD_LOCK.lock().unwrap();
        let dir = TempDir::new().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        fs::write(CONFIG_FILE, "{}").unwrap();
        let result = ensure_exists();

        std::env::set_current_dir(prev).unwrap();
        assert!(result.is_ok());
    }

    #[test]
    fn ensure_exists_errors_when_config_missing() {
        let _guard = CWD_LOCK.lock().unwrap();
        let dir = TempDir::new().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let result = ensure_exists();

        std::env::set_current_dir(prev).unwrap();
        match result {
            Err(CliError::MissingConfig(name)) => assert_eq!(name, CONFIG_FILE),
            other => panic!("expected MissingConfig, got {other:?}"),
        }
    }
}
