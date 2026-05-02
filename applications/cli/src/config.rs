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

/// Append `entry` on its own line to `.gitignore` in the current directory.
/// Creates the file if missing. No-op if the exact line already exists.
/// Returns `true` if the file was modified.
pub fn append_to_gitignore(entry: &str) -> Result<bool, CliError> {
    let path = Path::new(".gitignore");
    let existing = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(CliError::ConfigWrite(e)),
    };

    if existing.lines().any(|l| l.trim() == entry) {
        return Ok(false);
    }

    let needs_leading_newline = !existing.is_empty() && !existing.ends_with('\n');
    let mut new_contents = existing;
    if needs_leading_newline {
        new_contents.push('\n');
    }
    new_contents.push_str(entry);
    new_contents.push('\n');

    std::fs::write(path, new_contents).map_err(CliError::ConfigWrite)?;
    Ok(true)
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

/// Process-wide lock for tests that change the current working directory.
/// Library unit tests don't run with `--test-threads=1`, so any test that
/// `set_current_dir`s must hold this lock for the duration of the test.
#[cfg(test)]
pub(crate) static CWD_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

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

    #[test]
    fn append_to_gitignore_creates_file_if_missing() {
        let _guard = CWD_LOCK.lock().unwrap();
        let dir = TempDir::new().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        let added = append_to_gitignore(".whisperrc").unwrap();
        let contents = std::fs::read_to_string(".gitignore").unwrap();

        std::env::set_current_dir(prev).unwrap();
        assert!(added, "should report a change");
        assert!(contents.contains(".whisperrc"), "got: {contents}");
    }

    #[test]
    fn append_to_gitignore_idempotent_when_already_present() {
        let _guard = CWD_LOCK.lock().unwrap();
        let dir = TempDir::new().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        fs::write(".gitignore", "node_modules\n.whisperrc\n.env\n").unwrap();
        let added = append_to_gitignore(".whisperrc").unwrap();
        let contents = std::fs::read_to_string(".gitignore").unwrap();

        std::env::set_current_dir(prev).unwrap();
        assert!(!added, "should not report a change");
        assert_eq!(contents, "node_modules\n.whisperrc\n.env\n");
    }

    #[test]
    fn append_to_gitignore_handles_missing_trailing_newline() {
        let _guard = CWD_LOCK.lock().unwrap();
        let dir = TempDir::new().unwrap();
        let prev = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();

        fs::write(".gitignore", "node_modules").unwrap();
        let added = append_to_gitignore(".whisperrc").unwrap();
        let contents = std::fs::read_to_string(".gitignore").unwrap();

        std::env::set_current_dir(prev).unwrap();
        assert!(added);
        assert_eq!(contents, "node_modules\n.whisperrc\n");
    }
}
