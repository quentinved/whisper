use crate::{client::WhisperClient, config::WhisperConfig, crypto::CryptoContext, error::CliError};
use tracing::debug;

/// A loaded project session: config + crypto context + authenticated HTTP client.
pub struct Session {
    crypto: CryptoContext,
    client: WhisperClient,
}

impl Session {
    /// Load config from `.whisperrc`, derive encryption key and auth token.
    pub fn load() -> Result<Self, CliError> {
        debug!("Loading config");
        let config = WhisperConfig::load()?;
        debug!("Config loaded, server: {}", config.url);

        let crypto = CryptoContext::new(&config.passphrase, config.url.as_str())?;
        let client = WhisperClient::new(&config.url).with_auth(crypto.auth_token());

        Ok(Self { crypto, client })
    }

    pub fn crypto(&self) -> &CryptoContext {
        &self.crypto
    }

    pub fn client(&self) -> &WhisperClient {
        &self.client
    }
}
