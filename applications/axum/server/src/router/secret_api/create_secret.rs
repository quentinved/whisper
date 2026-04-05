use axum::{
    extract::{Query, State},
    response::Redirect,
    Form,
};
use serde::Deserialize;
use serde_json;
use std::sync::Arc;
use tracing::{error, info};
use whisper_core::{
    commands::shared_secret::create_secret::CreateSecret,
    values_object::shared_secret::secret_expiration::SecretExpiration,
};

use crate::{app_state::AppState, error::CustomError, source::Source};

const B64_URL_INVALID_EXPIRATION: &str = "RXhwaXJhdGlvbiBpcyBpbiB0aGUgcGFzdA";

pub async fn create_secret(
    State(app_state): State<Arc<AppState>>,
    Query(query): Query<CreateSecretQuery>,
    Form(create_secret_params): Form<CreateSecretParams>,
) -> Result<Redirect, CustomError> {
    let source = query.source.unwrap_or_default();
    let secret = create_secret_params.secret;
    let expiration = match SecretExpiration::try_from(create_secret_params.expiration) {
        Ok(expiration) => expiration,
        Err(err) => {
            error!("Create shared secret error: {}", err);
            return Ok(Redirect::to(
                format!("/?error={}", B64_URL_INVALID_EXPIRATION).as_str(),
            ));
        }
    };
    let self_destructed = create_secret_params.self_destruct.unwrap_or_default();
    let command = CreateSecret::new(secret, expiration, self_destructed)?;
    let secret_id = command
        .handle(&app_state.aes_gcm(), &app_state.shared_secret_repository())
        .await?;

    info!("Secret created id={} source={}", secret_id.value(), source);
    app_state.analytics().track(
        "secret_created",
        &source.to_string(),
        serde_json::json!({
            "self_destruct": self_destructed,
            "expiration": create_secret_params.expiration,
        }),
    );

    Ok(Redirect::to(
        format!("/?shared_secret_id={}", secret_id.value()).as_str(),
    ))
}

#[derive(Debug, Deserialize)]
pub struct CreateSecretQuery {
    pub source: Option<Source>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateSecretParams {
    pub secret: String,
    pub expiration: i64,
    pub self_destruct: Option<bool>,
}
