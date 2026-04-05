use std::sync::Arc;

use super::AnalyticsTracker;

#[derive(Debug)]
pub struct CompositeTracker {
    trackers: Vec<Arc<dyn AnalyticsTracker>>,
}

impl CompositeTracker {
    pub fn new(trackers: Vec<Arc<dyn AnalyticsTracker>>) -> Self {
        Self { trackers }
    }
}

impl AnalyticsTracker for CompositeTracker {
    fn track(&self, event: &str, source: &str, props: serde_json::Value) {
        for tracker in &self.trackers {
            tracker.track(event, source, props.clone());
        }
    }
}
