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
    /// Direct database access, for states the API refuses to create (e.g. a
    /// receipt whose secret already expired).
    pool: PgPool,
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
            pool.clone(),
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
            pool,
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
async fn apple_app_site_association_routes_key_links_to_the_ios_app() {
    let server = TestServer::start().await;

    let resp = server
        .client
        .get(server.url("/.well-known/apple-app-site-association"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers()["content-type"], "application/json");
    let body: serde_json::Value = resp.json().await.unwrap();
    let details = &body["applinks"]["details"][0];
    assert_eq!(
        details["appIDs"][0],
        "7WBNUC3KR9.com.quentinvedrenne.whisper"
    );
    // Only links carrying a key open in the app; Slack's server-encrypted
    // links have none and must keep opening on the web.
    for component in details["components"].as_array().unwrap() {
        assert_eq!(component["#"], "k=*");
    }
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
async fn secret_meta_is_non_consuming() {
    let server = TestServer::start().await;
    let expiration = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp();

    // Client-encrypted, self-destruct secret: hitting /meta must NOT burn it.
    let payload = base64_url::encode(&[0x42u8; 12 + 21 + 16]);
    let resp = server
        .client
        .post(server.url("/v1/ephemeral"))
        .json(&serde_json::json!({
            "payload": payload,
            "expiration": expiration,
            "self_destruct": true,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // First /meta: reports the flags.
    let resp = server
        .client
        .get(server.url(&format!("/secret/{}/meta", id)))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["exists"], true);
    assert_eq!(body["client_encrypted"], true);
    assert_eq!(body["self_destruct"], true);

    // Second /meta: secret was NOT consumed by the first lookup.
    let resp = server
        .client
        .get(server.url(&format!("/secret/{}/meta", id)))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["exists"], true);

    // And the consuming GET still finds it.
    let resp = server
        .client
        .get(server.url(&format!("/secret/{}", id)))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn secret_meta_returns_404_for_missing() {
    let server = TestServer::start().await;
    let resp = server
        .client
        .get(server.url(&format!("/secret/{}/meta", uuid::Uuid::new_v4())))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["exists"], false);
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
async fn footer_links_the_encryption_badge_and_the_repo_to_github() {
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

    // The badge shows where encryption happens, not the CLI docs.
    assert!(
        body.contains(
            r#"href="https://github.com/quentinved/Whisper/tree/main/adapters/aes-gcm-crypto""#
        ),
        "the AES-256-GCM badge must open the encryption crate"
    );
    assert!(
        body.contains(r#"href="https://github.com/quentinved/Whisper""#),
        "the footer must link to the repo"
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
async fn integrations_page_lists_ai_coding_agents() {
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

    assert!(body.contains("AI coding agents"), "missing AI agents card");
    assert!(
        body.contains("npx skills add quentinved/Whisper"),
        "card must show how to install the skill"
    );
    assert!(
        body.contains(r#"href="/docs/secrets#ai-agents""#),
        "card must link to the setup guide"
    );
}

#[tokio::test]
async fn cli_docs_cover_agents_run_and_pull() {
    let server = TestServer::start().await;
    let body = server
        .client
        .get(server.url("/docs/secrets"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert!(
        body.contains(r#"id="ai-agents""#),
        "integrations card links to #ai-agents"
    );
    assert!(body.contains("/plugin install whisper-secrets@whisper"));
    assert!(body.contains("whisper-secrets run -- npm start"));
    assert!(body.contains("whisper-secrets pull --yes"));
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

#[tokio::test]
async fn static_assets_are_cache_busted_with_version() {
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

    // CSS and JS carry a `?v=` query so a new release busts CDN/browser caches.
    // Version-agnostic on purpose — only the presence of the cache-buster matters.
    assert!(
        body.contains("/assets/css/modern.css?v="),
        "modern.css must be cache-busted with ?v="
    );
    assert!(
        body.contains("/assets/css/pages.css?v="),
        "pages.css must be cache-busted with ?v="
    );
    assert!(
        body.contains("/assets/create_secret.js?v="),
        "create_secret.js must be cache-busted with ?v="
    );
    assert!(
        body.contains("/assets/sent_secrets.js?v="),
        "sent_secrets.js must be cache-busted with ?v="
    );
}

async fn create_ephemeral_json(server: &TestServer, body: serde_json::Value) -> serde_json::Value {
    let resp = server
        .client
        .post(server.url("/v1/ephemeral"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    resp.json().await.unwrap()
}

async fn delivery_status(server: &TestServer, token: &str) -> reqwest::Response {
    server
        .client
        .get(server.url(&format!("/v1/status/{}", token)))
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn ephemeral_without_tracking_returns_no_status_token() {
    let server = TestServer::start().await;
    let expiration = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp();
    let body = create_ephemeral_json(
        &server,
        serde_json::json!({
            "payload": base64_url::encode(&[0x42u8; 12 + 21 + 16]),
            "expiration": expiration,
        }),
    )
    .await;
    assert!(body["id"].is_string());
    assert!(
        body.get("status_token").is_none(),
        "existing clients must see an unchanged response"
    );
}

#[tokio::test]
async fn delivery_status_goes_from_waiting_to_opened_and_survives_self_destruct() {
    let server = TestServer::start().await;
    let expiration = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp();
    let body = create_ephemeral_json(
        &server,
        serde_json::json!({
            "payload": base64_url::encode(&[0x42u8; 12 + 21 + 16]),
            "expiration": expiration,
            "self_destruct": true,
            "track_delivery": true,
        }),
    )
    .await;
    let id = body["id"].as_str().unwrap().to_string();
    let token = body["status_token"].as_str().unwrap().to_string();

    let resp = delivery_status(&server, &token).await;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.headers()["cache-control"], "no-store");
    let status: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(status["state"], "waiting");
    assert!(status["opened_at"].is_null());
    assert!(status["expires_at"].is_string());

    // A non-consuming peek is not an open.
    server
        .client
        .get(server.url(&format!("/secret/{}/meta", id)))
        .send()
        .await
        .unwrap();
    let status: serde_json::Value = delivery_status(&server, &token).await.json().await.unwrap();
    assert_eq!(status["state"], "waiting", "/meta must not count as opened");

    // The reveal burns the secret but the receipt outlives it.
    let resp = server
        .client
        .get(server.url(&format!("/secret/{}", id)))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let status: serde_json::Value = delivery_status(&server, &token).await.json().await.unwrap();
    assert_eq!(status["state"], "opened");
    assert!(status["opened_at"].is_string());

    // The status token is not a secret id.
    let resp = server
        .client
        .get(server.url(&format!("/secret/{}", token)))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delivery_status_unknown_is_404_and_malformed_is_400() {
    let server = TestServer::start().await;
    let resp = delivery_status(&server, &uuid::Uuid::new_v4().to_string()).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let resp = delivery_status(&server, "not-a-token").await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn delivery_status_keeps_the_first_open_time_of_a_reusable_secret() {
    let server = TestServer::start().await;
    let expiration = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp();
    let body = create_ephemeral_json(
        &server,
        serde_json::json!({
            "payload": base64_url::encode(&[0x42u8; 12 + 21 + 16]),
            "expiration": expiration,
            "self_destruct": false,
            "track_delivery": true,
        }),
    )
    .await;
    let id = body["id"].as_str().unwrap().to_string();
    let token = body["status_token"].as_str().unwrap().to_string();
    let reveal = || {
        server
            .client
            .get(server.url(&format!("/secret/{}", id)))
            .send()
    };

    assert_eq!(reveal().await.unwrap().status(), StatusCode::OK);
    let first: serde_json::Value = delivery_status(&server, &token).await.json().await.unwrap();
    assert_eq!(first["state"], "opened");

    // Past the 1-second resolution of the timestamps, so a moved time shows.
    tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
    assert_eq!(reveal().await.unwrap().status(), StatusCode::OK);
    let second: serde_json::Value = delivery_status(&server, &token).await.json().await.unwrap();
    assert_eq!(
        second["opened_at"], first["opened_at"],
        "a second reveal must not move the first-open time"
    );
}

/// Plants a receipt whose secret expired `days_ago` days ago — a state the
/// API refuses to create.
async fn insert_expired_receipt(server: &TestServer, days_ago: i32) -> String {
    let token = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO secret_receipts (status_token, secret_id, expiration)
         VALUES ($1, $2, NOW() - make_interval(days => $3))",
    )
    .bind(&token)
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(days_ago)
    .execute(&server.pool)
    .await
    .unwrap();
    token
}

#[tokio::test]
async fn expired_receipts_report_expired_then_are_purged_after_retention() {
    use whisper_core::commands::shared_secret::delete_expired_secrets::DeleteExpiredSecrets;
    use whisper_postgresql::shared_secrets_repository::PostgreSQLSharedSecretsRepository;

    let server = TestServer::start().await;
    let recent = insert_expired_receipt(&server, 1).await;
    let stale = insert_expired_receipt(&server, 8).await;

    for token in [&recent, &stale] {
        let resp = delivery_status(&server, token).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let status: serde_json::Value = resp.json().await.unwrap();
        assert_eq!(status["state"], "expired");
        assert!(status["opened_at"].is_null());
    }

    // The every-minute cleanup job, run once against the real database.
    let repository = PostgreSQLSharedSecretsRepository::new(server.pool.clone());
    DeleteExpiredSecrets::new()
        .handle(&repository)
        .await
        .unwrap();

    assert_eq!(
        delivery_status(&server, &stale).await.status(),
        StatusCode::NOT_FOUND,
        "a receipt 8 days past expiry must be purged"
    );
    assert_eq!(
        delivery_status(&server, &recent).await.status(),
        StatusCode::OK,
        "a receipt within the 7-day window must be kept"
    );
}
