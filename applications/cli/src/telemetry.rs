use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use tokio::task::JoinHandle;
use tracing::debug;
use uuid::Uuid;

pub const TELEMETRY_DIR: &str = ".whisper-secrets";
pub const TELEMETRY_FILE: &str = "telemetry_id";
pub const DO_NOT_TRACK_VAR: &str = "DO_NOT_TRACK";

const MIXPANEL_TOKEN: &str = "420f07e8459f212d7b4702a93a650abf";
const MIXPANEL_ENDPOINT: &str = "https://api-eu.mixpanel.com/track";
const EVENT_NAME: &str = "cli_command";

pub struct TelemetryId(Uuid);

impl TelemetryId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_str(&self) -> String {
        self.0.to_string()
    }

    pub fn load_or_create_at(path: &Path) -> Option<Self> {
        if let Ok(contents) = std::fs::read_to_string(path) {
            if let Ok(id) = Self::try_from(contents.trim()) {
                return Some(id);
            }
        }
        let id = Self::generate();
        persist_id(path, &id).ok()?;
        Some(id)
    }

    pub fn load_or_create() -> Option<Self> {
        let path = default_path()?;
        Self::load_or_create_at(&path)
    }
}

impl TryFrom<&str> for TelemetryId {
    type Error = uuid::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Uuid::parse_str(value).map(Self)
    }
}

pub fn track_command(command: &'static str, success: bool) -> Option<JoinHandle<()>> {
    if is_opted_out(std::env::var(DO_NOT_TRACK_VAR).ok().as_deref()) {
        return None;
    }

    let id = TelemetryId::load_or_create()?;
    let payload = build_event(&id, command, success);
    Some(tokio::spawn(send_event(payload)))
}

fn is_opted_out(env_value: Option<&str>) -> bool {
    matches!(env_value, Some(v) if !v.is_empty())
}

fn default_path() -> Option<PathBuf> {
    let home = home_dir()?;
    let dir = home.join(TELEMETRY_DIR);
    if let Err(e) = std::fs::create_dir_all(&dir) {
        debug!("telemetry: failed to create dir {:?}: {}", dir, e);
        return None;
    }
    Some(dir.join(TELEMETRY_FILE))
}

fn home_dir() -> Option<PathBuf> {
    let var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    std::env::var(var).ok().map(PathBuf::from)
}

fn persist_id(path: &Path, id: &TelemetryId) -> std::io::Result<()> {
    std::fs::write(path, id.as_str())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(path, perms)?;
    }
    Ok(())
}

fn build_event(id: &TelemetryId, command: &'static str, success: bool) -> Value {
    let time_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    json!([{
        "event": EVENT_NAME,
        "properties": {
            "token": MIXPANEL_TOKEN,
            "$insert_id": Uuid::new_v4().to_string(),
            "time": time_ms,
            "distinct_id": id.as_str(),
            "command": command,
            "success": success,
            "version": env!("CARGO_PKG_VERSION"),
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
        }
    }])
}

async fn send_event(payload: Value) {
    let client = reqwest::Client::new();
    match client
        .post(MIXPANEL_ENDPOINT)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
    {
        Err(e) => debug!("telemetry: network error: {:?}", e),
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            if !status.is_success() || body.trim() == "0" {
                debug!("telemetry: rejected status={} body={}", status, body);
            } else {
                debug!("telemetry: tracked body={}", body);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn generate_produces_valid_uuid() {
        let id = TelemetryId::generate();
        assert!(Uuid::parse_str(&id.as_str()).is_ok());
    }

    #[test]
    fn try_from_rejects_invalid_uuid() {
        assert!(TelemetryId::try_from("not-a-uuid").is_err());
    }

    #[test]
    fn try_from_roundtrips_valid_uuid() {
        let original = TelemetryId::generate();
        let parsed = TelemetryId::try_from(original.as_str().as_str()).unwrap();
        assert_eq!(original.as_str(), parsed.as_str());
    }

    #[test]
    fn load_or_create_at_creates_file_on_first_call() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("telemetry_id");
        assert!(!path.exists());

        let id = TelemetryId::load_or_create_at(&path).unwrap();
        assert!(path.exists());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), id.as_str());
    }

    #[test]
    fn load_or_create_at_reuses_existing_id() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("telemetry_id");

        let first = TelemetryId::load_or_create_at(&path).unwrap();
        let second = TelemetryId::load_or_create_at(&path).unwrap();
        assert_eq!(first.as_str(), second.as_str());
    }

    #[test]
    fn load_or_create_at_regenerates_if_file_corrupt() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("telemetry_id");
        std::fs::write(&path, "not-a-uuid").unwrap();

        let id = TelemetryId::load_or_create_at(&path).unwrap();
        assert!(Uuid::parse_str(&id.as_str()).is_ok());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), id.as_str());
    }

    #[cfg(unix)]
    #[test]
    fn load_or_create_at_sets_0600_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("telemetry_id");
        TelemetryId::load_or_create_at(&path).unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn is_opted_out_detects_set_value() {
        assert!(is_opted_out(Some("1")));
        assert!(is_opted_out(Some("true")));
        assert!(is_opted_out(Some("anything")));
    }

    #[test]
    fn is_opted_out_ignores_empty_or_unset() {
        assert!(!is_opted_out(None));
        assert!(!is_opted_out(Some("")));
    }

    #[test]
    fn build_event_contains_expected_properties() {
        let id = TelemetryId::generate();
        let payload = build_event(&id, "push", true);
        let event = payload.get(0).unwrap();
        assert_eq!(event["event"], EVENT_NAME);
        let props = &event["properties"];
        assert_eq!(props["command"], "push");
        assert_eq!(props["success"], true);
        assert_eq!(props["distinct_id"], id.as_str());
        assert_eq!(props["token"], MIXPANEL_TOKEN);
        assert_eq!(props["os"], std::env::consts::OS);
        assert_eq!(props["arch"], std::env::consts::ARCH);
        assert!(props.get("$insert_id").is_some());
        assert!(props.get("time").is_some());
        assert!(props.get("version").is_some());
    }
}
