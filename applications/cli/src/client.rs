use crate::error::CliError;
use reqwest::{header, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::debug;
use url::Url;

#[derive(Debug, Serialize)]
struct PutBody {
    payload: String,
}

#[derive(Debug, Deserialize)]
struct GetResponse {
    payload: String,
}

#[derive(Debug, Deserialize)]
pub struct EphemeralSecretResponse {
    pub secret: String,
    pub self_destruct: bool,
    #[serde(default)]
    pub client_encrypted: bool,
}

#[derive(Debug, Serialize)]
struct CreateEphemeralBody {
    payload: String,
    expiration: i64,
    self_destruct: bool,
}

#[derive(Debug, Deserialize)]
struct CreateEphemeralResponse {
    id: String,
}

pub struct WhisperClient {
    base_url: Url,
    http: reqwest::Client,
    auth_token: Option<String>,
}

impl WhisperClient {
    pub fn new(base_url: &Url) -> Self {
        let mut url = base_url.clone();
        let trimmed_path = url.path().trim_end_matches('/').to_string();
        url.set_path(&trimmed_path);
        Self {
            base_url: url,
            http: reqwest::Client::new(),
            auth_token: None,
        }
    }

    pub fn with_auth(mut self, auth_token: &str) -> Self {
        self.auth_token = Some(auth_token.to_string());
        self
    }

    fn url(&self, path: &str) -> Url {
        let mut url = self.base_url.clone();
        let base_path = url.path().trim_end_matches('/');
        url.set_path(&format!("{}/{}", base_path, path.trim_start_matches('/')));
        url
    }

    fn apply_auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.auth_token {
            Some(token) => req.header(header::AUTHORIZATION, format!("Bearer {}", token)),
            None => req,
        }
    }

    pub async fn put_secret(&self, id: &str, payload_base64: &str) -> Result<bool, CliError> {
        let url = self.url(&format!("v1/secrets/{}", id));
        debug!("PUT {}", url);
        let req = self.apply_auth(self.http.put(url.as_str()).json(&PutBody {
            payload: payload_base64.to_string(),
        }));
        let response = req.send().await.map_err(CliError::HttpRequest)?;

        debug!("PUT -> {}", response.status());
        match response.status() {
            StatusCode::CREATED => Ok(true),
            StatusCode::NO_CONTENT => Ok(false),
            StatusCode::BAD_REQUEST => Err(CliError::BadRequest),
            StatusCode::UNAUTHORIZED => Err(CliError::Unauthorized),
            StatusCode::FORBIDDEN => Err(CliError::Forbidden),
            StatusCode::TOO_MANY_REQUESTS => Err(CliError::RateLimited),
            other => Err(CliError::UnexpectedStatus(other.to_string())),
        }
    }

    pub async fn get_secret(&self, id: &str) -> Result<Option<String>, CliError> {
        let url = self.url(&format!("v1/secrets/{}", id));
        debug!("GET {}", url);
        let req = self.apply_auth(self.http.get(url.as_str()));
        let response = req.send().await.map_err(CliError::HttpRequest)?;

        debug!("GET -> {}", response.status());
        match response.status() {
            StatusCode::OK => {
                let body: GetResponse = response
                    .json()
                    .await
                    .map_err(|e| CliError::InvalidResponse(e.to_string()))?;
                Ok(Some(body.payload))
            }
            StatusCode::NOT_FOUND => Ok(None),
            StatusCode::UNAUTHORIZED => Err(CliError::Unauthorized),
            StatusCode::FORBIDDEN => Err(CliError::Forbidden),
            StatusCode::TOO_MANY_REQUESTS => Err(CliError::RateLimited),
            other => Err(CliError::UnexpectedStatus(other.to_string())),
        }
    }

    pub async fn get_ephemeral_secret(
        &self,
        id: &str,
    ) -> Result<Option<EphemeralSecretResponse>, CliError> {
        let mut url = self.url(&format!("secret/{}", id));
        url.query_pairs_mut().append_pair("source", "cli");
        debug!("GET {}", url);
        let response = self
            .http
            .get(url.as_str())
            .send()
            .await
            .map_err(CliError::HttpRequest)?;

        debug!("GET -> {}", response.status());
        match response.status() {
            StatusCode::OK => {
                let body: EphemeralSecretResponse = response
                    .json()
                    .await
                    .map_err(|e| CliError::InvalidResponse(e.to_string()))?;
                Ok(Some(body))
            }
            StatusCode::NOT_FOUND => Ok(None),
            StatusCode::TOO_MANY_REQUESTS => Err(CliError::RateLimited),
            other => Err(CliError::UnexpectedStatus(other.to_string())),
        }
    }

    /// Creates a zero-knowledge ephemeral secret: `payload_b64url` is
    /// `base64url(nonce ‖ ciphertext)` encrypted locally — the server never
    /// sees the plaintext or the key.
    pub async fn create_ephemeral_secret_v1(
        &self,
        payload_b64url: &str,
        expiration_timestamp: i64,
        self_destruct: bool,
    ) -> Result<String, CliError> {
        let mut url = self.url("v1/ephemeral");
        url.query_pairs_mut().append_pair("source", "cli");
        debug!("POST {}", url);
        let response = self
            .http
            .post(url.as_str())
            .json(&CreateEphemeralBody {
                payload: payload_b64url.to_string(),
                expiration: expiration_timestamp,
                self_destruct,
            })
            .send()
            .await
            .map_err(CliError::HttpRequest)?;

        debug!("POST -> {}", response.status());
        match response.status() {
            StatusCode::CREATED => {
                let body: CreateEphemeralResponse = response
                    .json()
                    .await
                    .map_err(|e| CliError::InvalidResponse(e.to_string()))?;
                Ok(body.id)
            }
            StatusCode::BAD_REQUEST => Err(CliError::BadRequest),
            StatusCode::NOT_FOUND => Err(CliError::ServerMissingZkEndpoint),
            StatusCode::TOO_MANY_REQUESTS => Err(CliError::RateLimited),
            other => Err(CliError::UnexpectedStatus(other.to_string())),
        }
    }

    /// Builds the shareable link for a zero-knowledge secret, with the key in
    /// the fragment (never sent to any server).
    pub fn ephemeral_share_url(&self, id: &str, key_b64url: &str) -> Url {
        let mut share_url = self.url("get_secret");
        share_url
            .query_pairs_mut()
            .append_pair("shared_secret_id", id);
        share_url.set_fragment(Some(&format!("k={}", key_b64url)));
        share_url
    }

    pub async fn delete_secret(&self, id: &str) -> Result<(), CliError> {
        let url = self.url(&format!("v1/secrets/{}", id));
        debug!("DELETE {}", url);
        let req = self.apply_auth(self.http.delete(url.as_str()));
        let response = req.send().await.map_err(CliError::HttpRequest)?;

        debug!("DELETE -> {}", response.status());
        match response.status() {
            StatusCode::NO_CONTENT | StatusCode::NOT_FOUND => Ok(()),
            StatusCode::UNAUTHORIZED => Err(CliError::Unauthorized),
            StatusCode::FORBIDDEN => Err(CliError::Forbidden),
            StatusCode::TOO_MANY_REQUESTS => Err(CliError::RateLimited),
            other => Err(CliError::UnexpectedStatus(other.to_string())),
        }
    }
}

/// Resolves the plaintext from an ephemeral-secret response: legacy plaintext
/// responses pass through as-is (backward compatibility with older
/// servers/links); zero-knowledge responses (`client_encrypted`) are decrypted
/// with the link's `#k=...` fragment key.
pub fn resolve_ephemeral_plaintext(
    resp: EphemeralSecretResponse,
    key: Option<&str>,
) -> Result<String, CliError> {
    if resp.client_encrypted {
        match key {
            Some(k) => crate::crypto::decrypt_ephemeral(k, &resp.secret).map_err(|e| match e {
                CliError::DecryptionError(_) => CliError::DecryptionError(
                    "the key after '#' is wrong or incomplete — make sure the entire link \
                     was copied (some apps truncate URLs at '#')"
                        .to_string(),
                ),
                other => other,
            }),
            None => Err(CliError::MissingFragmentKey),
        }
    } else {
        Ok(resp.secret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(secret: &str, client_encrypted: bool) -> EphemeralSecretResponse {
        EphemeralSecretResponse {
            secret: secret.to_string(),
            self_destruct: false,
            client_encrypted,
        }
    }

    #[test]
    fn legacy_plaintext_returned_as_is() {
        let resp = response("team-passphrase", false);
        let plaintext = resolve_ephemeral_plaintext(resp, None).unwrap();
        assert_eq!(plaintext, "team-passphrase");
    }

    #[test]
    fn legacy_plaintext_ignores_spurious_key() {
        let resp = response("team-passphrase", false);
        let plaintext = resolve_ephemeral_plaintext(resp, Some("unused-key")).unwrap();
        assert_eq!(plaintext, "team-passphrase");
    }

    #[test]
    fn zero_knowledge_payload_decrypts_with_fragment_key() {
        let (key, payload) = crate::crypto::encrypt_ephemeral("team-passphrase").unwrap();
        let resp = response(&payload, true);
        let plaintext = resolve_ephemeral_plaintext(resp, Some(&key)).unwrap();
        assert_eq!(plaintext, "team-passphrase");
    }

    #[test]
    fn zero_knowledge_payload_without_key_is_missing_fragment_key() {
        let (_key, payload) = crate::crypto::encrypt_ephemeral("team-passphrase").unwrap();
        let resp = response(&payload, true);
        let result = resolve_ephemeral_plaintext(resp, None);
        assert!(matches!(result, Err(CliError::MissingFragmentKey)));
    }

    #[test]
    fn zero_knowledge_payload_with_wrong_key_fails_with_truncation_hint() {
        let (_key, payload) = crate::crypto::encrypt_ephemeral("team-passphrase").unwrap();
        let (wrong_key, _) = crate::crypto::encrypt_ephemeral("other").unwrap();
        let resp = response(&payload, true);
        let err = resolve_ephemeral_plaintext(resp, Some(&wrong_key)).unwrap_err();
        match &err {
            CliError::DecryptionError(msg) => {
                assert!(
                    msg.contains("wrong or incomplete"),
                    "should explain the key is wrong or incomplete: {msg}"
                );
                assert!(
                    msg.contains("truncate"),
                    "should warn about URL truncation at '#': {msg}"
                );
            }
            other => panic!("expected DecryptionError, got {other:?}"),
        }
    }
}
