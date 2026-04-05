use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info};
use uuid::Uuid;

use super::AnalyticsTracker;

#[derive(Clone)]
pub struct MixpanelTracker {
    client: Client,
    token: Arc<str>,
}

impl std::fmt::Debug for MixpanelTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MixpanelTracker")
            .field("token", &"***")
            .finish()
    }
}

impl MixpanelTracker {
    pub fn new(token: String) -> Self {
        Self {
            client: Client::new(),
            token: Arc::from(token),
        }
    }
}

impl AnalyticsTracker for MixpanelTracker {
    fn track(&self, event: &str, source: &str, props: serde_json::Value) {
        let client = self.client.clone();
        let token = self.token.clone();
        let event = event.to_string();
        let source = source.to_string();

        tokio::spawn(async move {
            if std::env::var("RUST_ENV").as_deref() != Ok("production") {
                tracing::debug!("Mixpanel (dev): would track '{}' source={}", event, source);
                return;
            }

            let mut properties = json!({
                "token": token,
                "$insert_id": Uuid::new_v4().to_string(),
                "time": chrono::Utc::now().timestamp_millis(),
                "distinct_id": "server",
                "source": source,
            });
            if let (Some(obj), Some(extra)) = (properties.as_object_mut(), props.as_object()) {
                obj.extend(extra.clone());
            }
            let payload = json!([{ "event": event, "properties": properties }]);

            match client
                .post("https://api-eu.mixpanel.com/track")
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
            {
                Err(e) => error!("Mixpanel network error: {:?}", e),
                Ok(resp) => {
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    // Mixpanel always returns HTTP 200; body "1" = accepted, "0" = rejected
                    if !status.is_success() || body.trim() == "0" {
                        error!(
                            "Mixpanel rejected event '{}': status={} body={}",
                            event, status, body
                        );
                    } else {
                        info!(
                            "Mixpanel: tracked '{}' source={} body={}",
                            event, source, body
                        );
                    }
                }
            }
        });
    }
}
