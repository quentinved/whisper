use crate::{
    app_state::AppState, error::CustomError, html_templates::get_secret::GetSecretHtml,
    source::Source,
};
use axum::{
    extract::{Query, State},
    http::HeaderMap,
};
use std::sync::Arc;
use uuid::Uuid;
use whisper_core::{
    commands::shared_secret::get_secret_by_id::GetSecretById,
    values_object::shared_secret::secret_id::SecretId,
};

fn is_bot_request(headers: &HeaderMap) -> bool {
    headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|ua| {
            let ua_lower = ua.to_lowercase();
            ua_lower.contains("slackbot")
                || ua_lower.contains("slack-imgproxy")
                || ua_lower.contains("discordbot")
                || ua_lower.contains("telegrambot")
                || ua_lower.contains("whatsapp")
                || ua_lower.contains("twitterbot")
                || ua_lower.contains("facebookexternalhit")
                || ua_lower.contains("linkedinbot")
        })
        .unwrap_or(false)
}

pub async fn get_secret(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(params): Query<Params>,
) -> Result<GetSecretHtml, CustomError> {
    let Some(uuid) = params.shared_secret_id else {
        return Ok(GetSecretHtml::new(
            None,
            None,
            Some("Missing secret ID".to_string()),
        ));
    };

    if is_bot_request(&headers) {
        return Ok(GetSecretHtml::new(
            None,
            None,
            Some("Click the link to reveal your secret".to_string()),
        ));
    }

    let source = params.source.unwrap_or(Source::Web);
    let shared_secret_id = SecretId::new(uuid);
    let query = GetSecretById::new(shared_secret_id);
    let decrypted_secret = query
        .handle(&app_state.aes_gcm(), &app_state.shared_secret_repository())
        .await;
    match decrypted_secret {
        Ok(Some((secret, self_destruct))) => {
            app_state.analytics().track(
                "secret_retrieved",
                &source.to_string(),
                serde_json::Value::Null,
            );
            Ok(GetSecretHtml::new(Some(secret), Some(self_destruct), None))
        }
        Ok(None) => Ok(GetSecretHtml::new(
            None,
            None,
            Some("Secret not found".to_string()),
        )),
        Err(err) => Ok(GetSecretHtml::new(None, None, Some(err.to_string()))),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Params {
    pub shared_secret_id: Option<Uuid>,
    pub source: Option<Source>,
}
