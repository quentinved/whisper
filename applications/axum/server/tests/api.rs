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
async fn create_ephemeral_v1_roundtrips_payload_verbatim() {
    let server = TestServer::start().await;
    let expiration = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp();

    // base64url-no-pad `nonce[12] ‖ ciphertext` as produced by client-side
    // encryption — the server must store and return it verbatim, never decode
    // it into a plaintext.
    let payload = base64_url::encode(&[0x42u8; 12 + 21 + 16]);

    let resp = server
        .client
        .post(server.url("/v1/ephemeral"))
        .json(&serde_json::json!({
            "payload": payload,
            "expiration": expiration,
            "self_destruct": false,
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = resp.json().await.unwrap();
    let id = body["id"].as_str().unwrap().to_string();
    assert!(!id.is_empty());

    let resp = server
        .client
        .get(server.url(&format!("/secret/{}", id)))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["secret"], payload, "payload must round-trip verbatim");
    assert_eq!(body["client_encrypted"], true);
    assert_eq!(body["self_destruct"], false);
    assert_eq!(body["id"], id);
}

#[tokio::test]
async fn create_ephemeral_v1_rejects_invalid_base64_payload() {
    let server = TestServer::start().await;
    let expiration = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp();

    let resp = server
        .client
        .post(server.url("/v1/ephemeral"))
        .json(&serde_json::json!({
            "payload": "not base64url!!!",
            "expiration": expiration,
            "self_destruct": false,
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
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

#[tokio::test]
async fn homepage_has_absolute_seo_tags() {
    let server = TestServer::start().await;
    let body = server
        .client
        .get(server.url("/"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert!(
        body.contains(r#"<link rel="canonical" href="http://localhost/" />"#),
        "missing homepage canonical"
    );
    assert!(
        body.contains(r#"content="http://localhost/assets/og-banner.png""#),
        "og:image must be an absolute PNG URL"
    );
    assert!(
        body.contains(r#"property="og:url" content="http://localhost/""#),
        "missing og:url"
    );
    assert!(
        body.contains(r#""@type":"SoftwareApplication""#),
        "missing SoftwareApplication JSON-LD"
    );
    assert!(
        body.contains("Zero-Knowledge"),
        "homepage title should carry the zero-knowledge copy"
    );
}

#[tokio::test]
async fn integrations_page_has_its_own_canonical() {
    let server = TestServer::start().await;
    let body = server
        .client
        .get(server.url("/integrations"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert!(
        body.contains(r#"<link rel="canonical" href="http://localhost/integrations" />"#),
        "integrations page must have its own canonical (not the homepage's)"
    );
}

#[tokio::test]
async fn get_secret_shell_is_noindex() {
    let server = TestServer::start().await;
    let body = server
        .client
        .get(server.url("/get_secret?shared_secret_id=00000000-0000-0000-0000-000000000000"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert!(
        body.contains(r#"<meta name="robots" content="noindex, nofollow" />"#),
        "secret reveal page must stay noindex"
    );
}
