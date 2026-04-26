use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Secret '{name}' not found in .env.whisper")]
    SecretNotFound { name: String },

    #[error(
        "Secret '{name}' already exists. Use `whisper-secrets rotate {name}` to update its value."
    )]
    SecretAlreadyExists { name: String },

    #[error("Encryption failed: {reason}")]
    EncryptionFailed { reason: String },

    #[error("Decryption failed for '{name}': {reason}")]
    DecryptionFailed { name: String, reason: String },

    #[error("Decryption failed: {0}")]
    DecryptionError(String),

    #[error("Unsupported KDF version 0x{0:02x}. Try updating whisper-secrets.")]
    UnsupportedKdfVersion(u8),

    #[error("HTTP request failed")]
    HttpRequest(#[source] reqwest::Error),

    #[error("Server rejected the payload (empty or invalid)")]
    BadRequest,

    #[error("Unauthorized — missing or invalid credentials")]
    Unauthorized,

    #[error("Forbidden — wrong passphrase for this project")]
    Forbidden,

    #[error("Rate limited — try again later")]
    RateLimited,

    #[error("Secret '{name}' ({id}) not found on server")]
    NotFoundOnServer { name: String, id: String },

    #[error("Unexpected server status: {0}")]
    UnexpectedStatus(String),

    #[error("Invalid server response: {0}")]
    InvalidResponse(String),

    #[error("Invalid base64: {0}")]
    Base64(String),

    #[error("Failed to read .env.whisper")]
    EnvWhisperRead(#[source] std::io::Error),

    #[error("Malformed .env.whisper at line {line}: '{content}'")]
    EnvWhisperMalformed { line: usize, content: String },

    #[error("Failed to write .env.whisper")]
    EnvWhisperWrite(#[source] std::io::Error),

    #[error("Failed to read .env")]
    EnvRead(#[source] std::io::Error),

    #[error("Failed to write .env")]
    EnvWrite(#[source] std::io::Error),

    #[error("User input failed: {0}")]
    Input(String),

    #[error("No .env file found. The import command requires an existing .env file in the current directory.")]
    NoEnvFile,

    #[error("Secret not found — it may have expired or been deleted.")]
    SecretExpiredOrNotFound,

    #[error("Invalid duration '{0}'. Use formats like 30m, 1h, 24h, or 7d.")]
    InvalidDuration(String),

    #[error("No {0} found — run `whisper-secrets init` to create a project or `whisper-secrets join <link>` to join one.")]
    MissingConfig(String),

    #[error("Failed to write .whisperrc")]
    ConfigWrite(#[source] std::io::Error),

    #[error("Failed to read config")]
    ConfigRead(#[source] std::io::Error),

    #[error("Invalid config format")]
    ConfigParse(#[source] serde_json::Error),

    #[error("Missing 'passphrase' in .whisperrc")]
    MissingPassphrase,

    #[error("Secret '{name}' contains newlines. Multiline values are not supported in .env format. Base64-encode the value before storing, or manage it outside whisper-secrets.")]
    MultilineValue { name: String },

    #[error("Invalid share target: {0}")]
    InvalidShareTarget(String),

    #[error("Invalid URL in config: {0}")]
    ConfigInvalidUrl(#[from] url::ParseError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_config_error_mentions_init_and_join() {
        let err = CliError::MissingConfig(".whisperrc".to_string());
        let msg = err.to_string();
        assert!(msg.contains("init"), "error should mention init: {msg}");
        assert!(msg.contains("join"), "error should mention join: {msg}");
        assert!(
            msg.contains(".whisperrc"),
            "error should mention .whisperrc: {msg}"
        );
    }
}
