use crate::{app_state::AppState, error::CustomError, html_templates::index::IndexHtml};
use axum::extract::{Query, State};
use std::sync::Arc;

pub async fn index(
    State(app_state): State<Arc<AppState>>,
    Query(params): Query<Params>,
) -> Result<IndexHtml, CustomError> {
    match params.shared_secret_id {
        Some(shared_secret_id) => {
            let url = format!(
                "{}/get_secret?shared_secret_id={}",
                app_state.url(),
                shared_secret_id
            );
            Ok(IndexHtml::new(Some(url), None))
        }
        None => match params.error {
            Some(b64_err_msg) => {
                let err_msg_bytes =
                    base64_url::decode(&b64_err_msg).map_err(|_| CustomError::InternalError {
                        reason: "Failed to decode error message".to_string(),
                    })?;
                let err_msg =
                    String::from_utf8(err_msg_bytes).map_err(|_| CustomError::InternalError {
                        reason: "Failed to convert error message to UTF-8".to_string(),
                    })?;
                Ok(IndexHtml::new(None, Some(err_msg)))
            }
            None => Ok(IndexHtml::new(None, None)),
        },
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Params {
    pub shared_secret_id: Option<String>,
    pub error: Option<String>,
}
