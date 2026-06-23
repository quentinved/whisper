use crate::app_state::AppState;
use crate::html_templates::get_secret::GetSecretHtml;
use crate::html_templates::seo::SeoMeta;
use axum::extract::{Query, State};
use std::sync::Arc;
use uuid::Uuid;

/// Renders the click-to-reveal shell. The secret is NOT fetched here — it is
/// only consumed when the visitor clicks Reveal (via `GET /secret/:id`), so
/// link previews and crawlers can never burn a self-destruct secret.
pub async fn get_secret(
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<Params>,
) -> GetSecretHtml {
    let seo = SeoMeta::private(
        app_state.url(),
        "/get_secret",
        "Retrieve Secret — Whisper",
        "Securely retrieve your encrypted secret. This page is private and is not indexed.",
    );
    match params.shared_secret_id {
        Some(_) => GetSecretHtml::new(seo, None),
        None => GetSecretHtml::new(seo, Some("Missing secret ID".to_string())),
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Params {
    pub shared_secret_id: Option<Uuid>,
}
