use std::io::Write;
use std::sync::Arc;

use reqwest::StatusCode;
use sqlx::PgPool;
use tempfile::NamedTempFile;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use whisper_server::aes_key::load_aes_key;
use whisper_server::analytics::composite::CompositeTracker;
use whisper_server::analytics::AnalyticsTracker;
use whisper_server::app_state::AppState;

struct TestServer {
    base_url: String,
    client: reqwest::Client,
    _container: testcontainers::ContainerAsync<Postgres>,
    _aes_key_file: NamedTempFile,
}

impl TestServer {
    async fn start() -> Self {
        // Start PostgreSQL container
        let container = Postgres::default().start().await.unwrap();
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();
        let db_url = format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            host_port
        );

        // Create pool and apply migrations
        let pool = PgPool::connect(&db_url).await.unwrap();
        whisper_postgresql::migrations::apply_migrations(&pool)
            .await
            .unwrap();

        // Generate random AES key and write to temp file
        let aes_key: [u8; 32] = rand::random();
        let mut key_file = NamedTempFile::new().unwrap();
        key_file.write_all(&aes_key).unwrap();
        key_file.flush().unwrap();
        let secret_key = load_aes_key(key_file.path().to_str().unwrap()).unwrap();

        // Create AppState
        let analytics: Arc<dyn AnalyticsTracker> = Arc::new(CompositeTracker::new(vec![]));
        let app_state = Arc::new(AppState::new(
            pool,
            secret_key,
            "http://localhost".to_string(),
            None,
            analytics,
        ));

        // Build router and start server
        let router = whisper_server::router::app(app_state);
        let tcp_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = tcp_listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            axum::serve(tcp_listener, router).await.unwrap();
        });

        // Build reqwest client
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();

        TestServer {
            base_url: format!("http://127.0.0.1:{}", port),
            client,
            _container: container,
            _aes_key_file: key_file,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn create_secret(&self, secret: &str, expiration: i64, self_destruct: bool) -> String {
        let resp = self
            .client
            .post(self.url("/secret"))
            .form(&[
                ("secret", secret),
                ("expiration", &expiration.to_string()),
                ("self_destruct", &self_destruct.to_string()),
            ])
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::SEE_OTHER);
        let location = resp.headers().get("location").unwrap().to_str().unwrap();
        // location is "/?shared_secret_id={uuid}"
        let url = reqwest::Url::parse(&format!("http://localhost{}", location)).unwrap();
        url.query_pairs()
            .find(|(k, _)| k == "shared_secret_id")
            .unwrap()
            .1
            .to_string()
    }
}

#[tokio::test]
async fn health_check() {
    let server = TestServer::start().await;

    let resp = server
        .client
        .get(server.url("/health"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn create_secret_returns_redirect() {
    let server = TestServer::start().await;
    let expiration = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp();

    let resp = server
        .client
        .post(server.url("/secret"))
        .form(&[
            ("secret", "test-password"),
            ("expiration", &expiration.to_string()),
            ("self_destruct", "true"),
        ])
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::SEE_OTHER);
    let location = resp.headers().get("location").unwrap().to_str().unwrap();
    assert!(
        location.contains("shared_secret_id="),
        "redirect should contain shared_secret_id, got: {}",
        location
    );
}

#[tokio::test]
async fn create_and_retrieve_secret() {
    let server = TestServer::start().await;
    let expiration = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp();

    let secret_id = server
        .create_secret("my-secret-value", expiration, false)
        .await;

    let resp = server
        .client
        .get(server.url(&format!("/secret/{}", secret_id)))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["secret"], "my-secret-value");
    assert_eq!(body["self_destruct"], false);
    assert_eq!(body["id"], secret_id);
}

#[tokio::test]
async fn retrieve_nonexistent_secret() {
    let server = TestServer::start().await;
    let random_uuid = uuid::Uuid::new_v4();

    let resp = server
        .client
        .get(server.url(&format!("/secret/{}", random_uuid)))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn self_destruct_deletes_after_first_read() {
    let server = TestServer::start().await;
    let expiration = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp();

    let secret_id = server
        .create_secret("one-time-secret", expiration, true)
        .await;

    // First read: should return the secret
    let resp1 = server
        .client
        .get(server.url(&format!("/secret/{}", secret_id)))
        .send()
        .await
        .unwrap();
    assert_eq!(resp1.status(), StatusCode::OK);
    let body: serde_json::Value = resp1.json().await.unwrap();
    assert_eq!(body["secret"], "one-time-secret");
    assert_eq!(body["self_destruct"], true);

    // Second read: should be gone
    let resp2 = server
        .client
        .get(server.url(&format!("/secret/{}", secret_id)))
        .send()
        .await
        .unwrap();
    assert_eq!(resp2.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_secret_too_large() {
    let server = TestServer::start().await;
    let expiration = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp();
    let large_secret = "x".repeat(64 * 1024 + 1); // > 64KB

    let resp = server
        .client
        .post(server.url("/secret"))
        .form(&[
            ("secret", large_secret.as_str()),
            ("expiration", &expiration.to_string()),
            ("self_destruct", "false"),
        ])
        .send()
        .await
        .unwrap();

    // The server returns a 500 (Internal Server Error) for validation failures
    // that bubble up through CustomError, or a redirect with error
    assert!(
        resp.status().is_client_error() || resp.status().is_server_error(),
        "expected error status, got: {}",
        resp.status()
    );
}
