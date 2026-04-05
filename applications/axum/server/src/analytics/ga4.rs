use reqwest::Client;
use serde_json::json;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

use super::AnalyticsTracker;

#[derive(Clone)]
pub struct GA4Tracker {
    client: Client,
    measurement_id: Arc<str>,
    api_secret: Arc<str>,
}

impl std::fmt::Debug for GA4Tracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GA4Tracker")
            .field("measurement_id", &self.measurement_id)
            .field("api_secret", &"***")
            .finish()
    }
}

impl GA4Tracker {
    pub fn new(measurement_id: String, api_secret: String) -> Self {
        Self {
            client: Client::new(),
            measurement_id: Arc::from(measurement_id),
            api_secret: Arc::from(api_secret),
        }
    }
}

impl AnalyticsTracker for GA4Tracker {
    fn track(&self, event: &str, source: &str, props: serde_json::Value) {
        let client = self.client.clone();
        let measurement_id = self.measurement_id.clone();
        let api_secret = self.api_secret.clone();
        let event = event.to_string();
        let source = source.to_string();

        tokio::spawn(async move {
            let is_dev = std::env::var("RUST_ENV").as_deref() != Ok("production");
            let endpoint = if is_dev { "debug/mp" } else { "mp" };
            let url = format!(
                "https://www.google-analytics.com/{}/collect?measurement_id={}&api_secret={}",
                endpoint, measurement_id, api_secret
            );

            let mut params = json!({ "source": source });
            if let (Some(obj), Some(extra)) = (params.as_object_mut(), props.as_object()) {
                obj.extend(extra.clone());
            }
            let payload = json!({
                "client_id": Uuid::new_v4().to_string(),
                "events": [{ "name": event, "params": params }],
            });

            if let Err(e) = client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await
            {
                error!("GA4 tracking error: {:?}", e);
            }
        });
    }
}
