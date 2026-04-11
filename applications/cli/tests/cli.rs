use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use sqlx::PgPool;
use tempfile::{NamedTempFile, TempDir};
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use url::Url;
use whisper_server::aes_key::load_aes_key;
use whisper_server::analytics::composite::CompositeTracker;
use whisper_server::analytics::AnalyticsTracker;
use whisper_server::app_state::AppState;

use whisper_secrets::client::WhisperClient;

struct TestEnv {
    server_url: Url,
    work_dir: TempDir,
    _container: testcontainers::ContainerAsync<Postgres>,
    _aes_key_file: NamedTempFile,
}

impl TestEnv {
    async fn start() -> Self {
        // 1. Start PostgreSQL container
        let container = Postgres::default().start().await.unwrap();
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();
        let db_url = format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            host_port
        );

        // 2. Create pool and apply migrations
        let pool = PgPool::connect(&db_url).await.unwrap();
        whisper_postgresql::migrations::apply_migrations(&pool)
            .await
            .unwrap();

        // 3. Generate random AES key
        let aes_key: [u8; 32] = rand::random();
        let mut key_file = NamedTempFile::new().unwrap();
        key_file.write_all(&aes_key).unwrap();
        key_file.flush().unwrap();
        let secret_key = load_aes_key(key_file.path().to_str().unwrap()).unwrap();

        // 4. Create AppState and boot server
        let analytics: Arc<dyn AnalyticsTracker> =
            Arc::new(CompositeTracker::new(vec![]));
        let app_state = Arc::new(AppState::new(
            pool,
            secret_key,
            "http://localhost".to_string(),
            None,
            analytics,
        ));
        let router = whisper_server::router::app(app_state);
        let tcp_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .unwrap();
        let port = tcp_listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            axum::serve(tcp_listener, router).await.unwrap();
        });

        let server_url = Url::parse(&format!("http://127.0.0.1:{}", port)).unwrap();
        let work_dir = TempDir::new().unwrap();

        // Give the server a moment to start accepting connections
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        TestEnv {
            server_url,
            work_dir,
            _container: container,
            _aes_key_file: key_file,
        }
    }

    /// Set CWD to the test's working directory
    fn enter_work_dir(&self) {
        std::env::set_current_dir(self.work_dir.path()).unwrap();
    }

    /// Create a second working directory (e.g. for teammate scenario)
    fn create_second_dir(&self) -> TempDir {
        TempDir::new().unwrap()
    }

    /// Helper: init a project in the current directory
    async fn init(&self) {
        whisper_secrets::commands::init::run(Some(self.server_url.as_str()), false)
            .await
            .unwrap();
    }

    /// Helper: push a secret using library calls (bypasses dialoguer prompt)
    async fn push_secret(&self, name: &str, value: &str) {
        let session = whisper_secrets::session::Session::load().unwrap();
        let payload = session.crypto().encrypt(value).unwrap();
        let payload_b64 = base64_url::encode(&payload);
        let id = uuid::Uuid::new_v4().to_string();
        session.client().put_secret(&id, &payload_b64).await.unwrap();
        whisper_secrets::env_whisper::set(name, &id).unwrap();
    }

    /// Helper: pull secrets (deletes .env first to avoid overwrite prompt)
    async fn pull(&self) {
        let env_path = Path::new(".env");
        if env_path.exists() {
            std::fs::remove_file(env_path).unwrap();
        }
        whisper_secrets::commands::pull::run().await.unwrap();
    }

    /// Helper: read .env file contents as key=value pairs
    fn read_env(&self) -> std::collections::HashMap<String, String> {
        let content = std::fs::read_to_string(".env").unwrap_or_default();
        content
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .filter_map(|l| l.split_once('='))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    /// Helper: read .whisperrc config
    fn read_config(&self) -> serde_json::Value {
        let content = std::fs::read_to_string(".whisperrc").unwrap();
        serde_json::from_str(&content).unwrap()
    }
}


#[tokio::test]
async fn init_creates_config_and_shareable_link() {
    let env = TestEnv::start().await;
    env.enter_work_dir();

    // Init project
    env.init().await;

    // Assert .whisperrc exists with url and passphrase
    let config = env.read_config();
    assert_eq!(config["url"], env.server_url.as_str());
    assert!(config["passphrase"].as_str().unwrap().len() > 10);

    let passphrase = config["passphrase"].as_str().unwrap();

    // Test the ephemeral secret sharing flow:
    // Create a new ephemeral secret with the passphrase
    let client = WhisperClient::new(&env.server_url);
    let expiration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + 3600;
    let share_url = client
        .create_ephemeral_secret(passphrase, expiration, false)
        .await
        .unwrap();

    // Extract secret ID from share URL
    let secret_id = share_url
        .query_pairs()
        .find(|(k, _)| k == "shared_secret_id")
        .unwrap()
        .1
        .to_string();

    // First GET — should succeed
    let result1 = client.get_ephemeral_secret(&secret_id).await.unwrap();
    assert!(result1.is_some());
    assert_eq!(result1.unwrap().secret, passphrase);

    // Second GET — should still succeed (not self-destruct)
    let result2 = client.get_ephemeral_secret(&secret_id).await.unwrap();
    assert!(result2.is_some());
    assert_eq!(result2.unwrap().secret, passphrase);
}

#[tokio::test]
async fn push_and_pull_single_secret() {
    let env = TestEnv::start().await;
    env.enter_work_dir();
    env.init().await;

    // Push a secret
    env.push_secret("DATABASE_URL", "postgres://localhost/mydb").await;

    // Assert .env.whisper has the entry
    let whisper_entries = whisper_secrets::env_whisper::read().unwrap();
    assert!(whisper_entries.contains_key("DATABASE_URL"));

    // Pull and verify .env
    env.pull().await;
    let env_vars = env.read_env();
    assert_eq!(env_vars.get("DATABASE_URL").unwrap(), "postgres://localhost/mydb");
}

#[tokio::test]
async fn push_pull_multiple_secrets() {
    let env = TestEnv::start().await;
    env.enter_work_dir();
    env.init().await;

    // Push 3 secrets
    env.push_secret("DB_URL", "postgres://localhost/db").await;
    env.push_secret("API_KEY", "sk-1234567890").await;
    env.push_secret("SECRET_TOKEN", "tok_abcdef").await;

    // Pull and verify all 3
    env.pull().await;
    let env_vars = env.read_env();
    assert_eq!(env_vars.len(), 3);
    assert_eq!(env_vars["DB_URL"], "postgres://localhost/db");
    assert_eq!(env_vars["API_KEY"], "sk-1234567890");
    assert_eq!(env_vars["SECRET_TOKEN"], "tok_abcdef");
}

#[tokio::test]
async fn remove_deletes_secret() {
    let env = TestEnv::start().await;
    env.enter_work_dir();
    env.init().await;

    // Push a secret
    env.push_secret("TO_DELETE", "sensitive-value").await;
    assert!(whisper_secrets::env_whisper::get("TO_DELETE").unwrap().is_some());

    // Remove it
    whisper_secrets::commands::remove::run("TO_DELETE").await.unwrap();

    // Assert gone from .env.whisper
    assert!(whisper_secrets::env_whisper::get("TO_DELETE").unwrap().is_none());

    // Pull — .env should not contain TO_DELETE
    env.pull().await;
    let env_path = std::path::Path::new(".env");
    if env_path.exists() {
        let env_vars = env.read_env();
        assert!(!env_vars.contains_key("TO_DELETE"));
    }
    // If .env doesn't exist at all, that's also correct (no secrets left)
}

#[tokio::test]
async fn import_existing_env_file() {
    let env = TestEnv::start().await;
    env.enter_work_dir();
    env.init().await;

    // Write a .env file with 3 entries
    std::fs::write(".env", "A=1\nB=2\nC=3\n").unwrap();

    // Import
    whisper_secrets::commands::import::run().await.unwrap();

    // Assert .env.whisper has 3 entries
    let whisper_entries = whisper_secrets::env_whisper::read().unwrap();
    assert_eq!(whisper_entries.len(), 3);
    assert!(whisper_entries.contains_key("A"));
    assert!(whisper_entries.contains_key("B"));
    assert!(whisper_entries.contains_key("C"));

    // Delete .env, pull, verify recovery
    std::fs::remove_file(".env").unwrap();
    env.pull().await;
    let env_vars = env.read_env();
    assert_eq!(env_vars.len(), 3);
    assert_eq!(env_vars["A"], "1");
    assert_eq!(env_vars["B"], "2");
    assert_eq!(env_vars["C"], "3");
}

#[tokio::test]
async fn teammate_pulls_shared_secrets() {
    let env = TestEnv::start().await;

    // === Developer 1: init + push secrets ===
    env.enter_work_dir();
    env.init().await;

    env.push_secret("DB_PASSWORD", "s3cret").await;
    env.push_secret("API_KEY", "key-xyz").await;

    // Read dev1's config and .env.whisper
    let config = env.read_config();
    let passphrase = config["passphrase"].as_str().unwrap().to_string();
    let env_whisper_content = std::fs::read_to_string(".env.whisper").unwrap();

    // === Developer 2: new directory, same passphrase ===
    let dir2 = env.create_second_dir();
    std::env::set_current_dir(dir2.path()).unwrap();

    // Write .whisperrc with same passphrase and server URL
    let config2 = serde_json::json!({
        "url": env.server_url.as_str(),
        "passphrase": passphrase,
    });
    std::fs::write(".whisperrc", serde_json::to_string_pretty(&config2).unwrap() + "\n").unwrap();

    // Copy .env.whisper (simulates git clone)
    std::fs::write(".env.whisper", &env_whisper_content).unwrap();

    // Pull secrets
    whisper_secrets::commands::pull::run().await.unwrap();

    // Assert dev2 gets the same secrets
    let content = std::fs::read_to_string(".env").unwrap();
    let env_vars: std::collections::HashMap<String, String> = content
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    assert_eq!(env_vars.len(), 2);
    assert_eq!(env_vars["DB_PASSWORD"], "s3cret");
    assert_eq!(env_vars["API_KEY"], "key-xyz");
}

#[tokio::test]
async fn share_and_get_secret() {
    let env = TestEnv::start().await;
    env.enter_work_dir();

    // Create ephemeral secret via WhisperClient (bypasses dialoguer prompt in share::run)
    let client = WhisperClient::new(&env.server_url);
    let expiration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + 3600;
    let share_url = client
        .create_ephemeral_secret("ephemeral-secret-value", expiration, false)
        .await
        .unwrap();

    // Extract secret ID and retrieve via WhisperClient
    let secret_id = share_url
        .query_pairs()
        .find(|(k, _)| k == "shared_secret_id")
        .unwrap()
        .1
        .to_string();

    let result = client.get_ephemeral_secret(&secret_id).await.unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().secret, "ephemeral-secret-value");
}
