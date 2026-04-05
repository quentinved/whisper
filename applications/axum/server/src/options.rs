use clap::Parser;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Options {
    /// TCP port to listen on
    #[arg(short, long, default_value_t = 1212)]
    pub port: u16,

    /// IP address to listen on
    #[arg(short, long, default_value_t = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))]
    pub listen_addr: IpAddr,

    /// PostgreSQL connection url (e.g., postgres://user:pass@host/db)
    #[arg(short, long, env = "DATABASE_URL")]
    pub url_postgresql: String,

    /// Path to the AES-256 key file (32 bytes)
    #[arg(short, long, env = "AES_KEY_PATH", default_value = "aes_key.bin")]
    pub aes_key_path: String,

    /// Public base URL for shared secret links
    #[arg(short, long, env = "BASE_URL")]
    pub base_url: Option<String>,

    /// Slack signing secret for slash command verification (enables Slack integration)
    #[arg(long, env = "SLACK_SIGNING_SECRET")]
    pub slack_signing_secret: Option<String>,

    /// Mixpanel project token for server-side event tracking (enables Mixpanel integration)
    #[arg(long, env = "MIXPANEL_TOKEN")]
    pub mixpanel_token: Option<String>,

    /// GA4 Measurement ID for server-side event tracking (e.g., G-XXXXXXXXXX)
    #[arg(long, env = "GA4_MEASUREMENT_ID")]
    pub ga4_measurement_id: Option<String>,

    /// GA4 Measurement Protocol API secret (enables GA4 server-side tracking)
    #[arg(long, env = "GA4_API_SECRET")]
    pub ga4_api_secret: Option<String>,
}
