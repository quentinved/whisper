pub mod duration;
pub mod signature;
pub mod whisper_command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct SlackResponse {
    pub response_type: String,
    pub text: String,
}

impl SlackResponse {
    pub fn ephemeral(text: impl Into<String>) -> Self {
        Self {
            response_type: "ephemeral".to_string(),
            text: text.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SlackSlashCommandPayload {
    pub text: String,
    pub user_name: String,
    pub team_id: String,
}
