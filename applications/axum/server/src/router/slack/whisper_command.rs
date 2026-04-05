use axum::{extract::State, http::StatusCode, Form, Json};
use chrono::Utc;
use serde_json;
use std::sync::Arc;
use tracing::{error, info};
use whisper_core::{
    commands::shared_secret::create_secret::CreateSecret,
    values_object::shared_secret::secret_expiration::SecretExpiration,
};

use crate::app_state::AppState;

use super::{duration::parse_duration, SlackResponse, SlackSlashCommandPayload};

const DEFAULT_DURATION_SECONDS: i64 = 3600; // 1 hour

pub async fn handle_whisper(
    State(app_state): State<Arc<AppState>>,
    Form(payload): Form<SlackSlashCommandPayload>,
) -> (StatusCode, Json<SlackResponse>) {
    info!(
        "Slack /whisper command from user={} team={}",
        payload.user_name, payload.team_id
    );

    let text = payload.text.trim();
    if text.is_empty() {
        return (
            StatusCode::OK,
            Json(SlackResponse::ephemeral(
                "Usage: `/whisper <secret> [30m|1h|24h|7d] [false = no self-destruct]`\n\
                 Max: 7 days. Default: 1 hour, self-destructs after first view.",
            )),
        );
    }

    // Parse: trailing options (duration, false flag), rest is the secret
    let (secret_text, duration_seconds, self_destruct) = parse_command(text);

    if secret_text.is_empty() {
        return (
            StatusCode::OK,
            Json(SlackResponse::ephemeral(
                "Error: Secret text cannot be empty.",
            )),
        );
    }

    let expiration_timestamp = Utc::now().timestamp() + duration_seconds;
    let expiration = match SecretExpiration::try_from(expiration_timestamp) {
        Ok(exp) => exp,
        Err(err) => {
            error!("Failed to create expiration: {}", err);
            return (
                StatusCode::OK,
                Json(SlackResponse::ephemeral(
                    "Error: Invalid expiration duration.",
                )),
            );
        }
    };

    let command = match CreateSecret::new(secret_text.to_string(), expiration, self_destruct) {
        Ok(cmd) => cmd,
        Err(err) => {
            return (
                StatusCode::OK,
                Json(SlackResponse::ephemeral(format!("Error: {}", err))),
            );
        }
    };
    match command
        .handle(&app_state.aes_gcm(), &app_state.shared_secret_repository())
        .await
    {
        Ok(secret_id) => {
            app_state.analytics().track(
                "secret_created",
                "slack",
                serde_json::json!({
                    "self_destruct": self_destruct,
                    "expiration": duration_seconds,
                }),
            );

            let share_url = format!(
                "{}/get_secret?shared_secret_id={}",
                app_state.url(),
                secret_id.value()
            );
            let duration_display = format_duration(duration_seconds);
            let destruct_note = if self_destruct {
                "Self-destructs after first view."
            } else {
                "Can be viewed multiple times until expiration."
            };
            (
                StatusCode::OK,
                Json(SlackResponse::ephemeral(format!(
                    "Secret created! Share this link:\n{}\n\n\
                     Expires in {}. {}",
                    share_url, duration_display, destruct_note
                ))),
            )
        }
        Err(err) => {
            error!("Failed to create secret via Slack: {:?}", err);
            (
                StatusCode::OK,
                Json(SlackResponse::ephemeral(
                    "Error: Failed to create secret. Please try again.",
                )),
            )
        }
    }
}

/// Parses the command text into (secret, duration_in_seconds, self_destruct).
/// Scans trailing tokens for a duration (e.g., "1h", "30m") and/or "false" flag.
/// "false" disables self-destruct. Default: 1h duration, self-destruct enabled.
fn parse_command(text: &str) -> (&str, i64, bool) {
    let mut self_destruct = true;
    let mut duration_seconds = DEFAULT_DURATION_SECONDS;
    let mut remaining = text;

    // Check last token for "false"
    if let Some((before, last)) = remaining.rsplit_once(' ') {
        if last.eq_ignore_ascii_case("false") {
            self_destruct = false;
            remaining = before.trim();
        }
    }

    // Check last token for duration
    if let Some((before, last)) = remaining.rsplit_once(' ') {
        if let Some(seconds) = parse_duration(last) {
            duration_seconds = seconds;
            remaining = before.trim();
        }
    }

    (remaining, duration_seconds, self_destruct)
}

fn format_duration(seconds: i64) -> String {
    if seconds >= 86400 && seconds % 86400 == 0 {
        format!("{} day(s)", seconds / 86400)
    } else if seconds >= 3600 && seconds % 3600 == 0 {
        format!("{} hour(s)", seconds / 3600)
    } else {
        format!("{} minute(s)", seconds / 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_only() {
        let (secret, dur, sd) = parse_command("my password");
        assert_eq!(secret, "my password");
        assert_eq!(dur, DEFAULT_DURATION_SECONDS);
        assert!(sd);
    }

    #[test]
    fn test_secret_with_duration() {
        let (secret, dur, sd) = parse_command("my password 24h");
        assert_eq!(secret, "my password");
        assert_eq!(dur, 86400);
        assert!(sd);
    }

    #[test]
    fn test_secret_with_minutes() {
        let (secret, dur, sd) = parse_command("api_key_123 30m");
        assert_eq!(secret, "api_key_123");
        assert_eq!(dur, 1800);
        assert!(sd);
    }

    #[test]
    fn test_secret_with_days() {
        let (secret, dur, sd) = parse_command("ssh-key-content 7d");
        assert_eq!(secret, "ssh-key-content");
        assert_eq!(dur, 604800);
        assert!(sd);
    }

    #[test]
    fn test_single_word_secret() {
        let (secret, dur, sd) = parse_command("password123");
        assert_eq!(secret, "password123");
        assert_eq!(dur, DEFAULT_DURATION_SECONDS);
        assert!(sd);
    }

    #[test]
    fn test_duration_like_word_not_valid_format() {
        let (secret, dur, sd) = parse_command("password 10s");
        assert_eq!(secret, "password 10s");
        assert_eq!(dur, DEFAULT_DURATION_SECONDS);
        assert!(sd);
    }

    #[test]
    fn test_false_flag() {
        let (secret, dur, sd) = parse_command("my secret false");
        assert_eq!(secret, "my secret");
        assert_eq!(dur, DEFAULT_DURATION_SECONDS);
        assert!(!sd);
    }

    #[test]
    fn test_duration_and_false() {
        let (secret, dur, sd) = parse_command("toto 1h false");
        assert_eq!(secret, "toto");
        assert_eq!(dur, 3600);
        assert!(!sd);
    }

    #[test]
    fn test_false_case_insensitive() {
        let (secret, _, sd) = parse_command("my secret FALSE");
        assert_eq!(secret, "my secret");
        assert!(!sd);
    }

    #[test]
    fn test_format_duration_days() {
        assert_eq!(format_duration(86400), "1 day(s)");
        assert_eq!(format_duration(604800), "7 day(s)");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration(3600), "1 hour(s)");
        assert_eq!(format_duration(7200), "2 hour(s)");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(1800), "30 minute(s)");
        assert_eq!(format_duration(60), "1 minute(s)");
    }
}
