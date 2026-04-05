pub mod composite;
pub mod ga4;
pub mod mixpanel;

pub trait AnalyticsTracker: Send + Sync + std::fmt::Debug {
    fn track(&self, event: &str, source: &str, props: serde_json::Value);
}
