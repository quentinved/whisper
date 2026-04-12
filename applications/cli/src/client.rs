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

    pub async fn create_ephemeral_secret(
        &self,
        secret: &str,
        expiration_timestamp: i64,
        self_destruct: bool,
    ) -> Result<Url, CliError> {
        let mut url = self.url("secret");
        url.query_pairs_mut().append_pair("source", "cli");
        debug!("POST {}", url);
        let response = self
            .http
            .post(url.as_str())
            .form(&[
                ("secret", secret),
                ("expiration", &expiration_timestamp.to_string()),
                ("self_destruct", &self_destruct.to_string()),
            ])
            .send()
            .await
            .map_err(CliError::HttpRequest)?;

        let status = response.status();
        debug!("POST -> {}", status);
        if !status.is_success() && !status.is_redirection() {
            return Err(CliError::UnexpectedStatus(status.to_string()));
        }

        // The server redirects to /?shared_secret_id=UUID
        let final_url = response.url();
        let secret_id = final_url
            .query_pairs()
            .find(|(k, _)| k == "shared_secret_id")
            .map(|(_, v)| v.to_string())
            .ok_or_else(|| CliError::InvalidResponse("No secret ID in redirect".to_string()))?;

        let mut share_url = self.url("get_secret");
        share_url
            .query_pairs_mut()
            .append_pair("shared_secret_id", &secret_id);
        Ok(share_url)
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
