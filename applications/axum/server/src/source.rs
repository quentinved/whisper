use serde::Deserialize;
use std::fmt;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Web,
    Cli,
    Slack,
    Discord,
    Raycast,
    #[default]
    #[serde(other)]
    Unknown,
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Source::Web => write!(f, "web"),
            Source::Cli => write!(f, "cli"),
            Source::Slack => write!(f, "slack"),
            Source::Discord => write!(f, "discord"),
            Source::Raycast => write!(f, "raycast"),
            Source::Unknown => write!(f, "unknown"),
        }
    }
}
