use crate::error::CliError;
use serde::Deserialize;
use std::path::Path;
use url::Url;

pub const DEFAULT_URL: &str = "https://whisper.quentinvedrenne.com";
pub const CONFIG_FILE: &str = ".whisperrc";

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
