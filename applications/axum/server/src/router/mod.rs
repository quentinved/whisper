use crate::{
    app_state::AppState,
    html_templates::{
        docs::DocsSecretsHtml,
        integrations::IntegrationsHtml,
        legal::{PrivacyHtml, TermsHtml},
    },
};
use axum::{
    extract::{Path, State},
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post, put},
    Router,
};
use rust_embed::Embed;
use secret_api::{create_secret::create_secret, get_secret_by_id::get_secret_by_id};
use std::sync::Arc;
use tower_http::set_header::SetResponseHeaderLayer;

mod managed_secret_api;
mod secret_api;
mod slack;
mod views;

#[derive(Embed)]
#[folder = "assets/"]
struct Assets;

async fn serve_asset(Path(path): Path<String>) -> impl IntoResponse {
    match Assets::get(&path) {
        Some(file) => {
            let mime_type = file.metadata.mimetype();
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime_type.to_string())],
                file.data.into_owned(),
            )
                .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

// Health check endpoint
async fn health() -> StatusCode {
    StatusCode::OK
}

async fn robots_txt(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let body = format!(
        "User-agent: *\nAllow: /\nDisallow: /get_secret\nDisallow: /secret/\n\nSitemap: {}/sitemap.xml\n",
        state.url().trim_end_matches('/')
    );
    ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body)
}

async fn sitemap_xml(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let base = state.url().trim_end_matches('/');
    let body = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>{}/</loc><changefreq>weekly</changefreq><priority>1.0</priority></url>
  <url><loc>{}/integrations</loc><changefreq>monthly</changefreq><priority>0.8</priority></url>
  <url><loc>{}/privacy</loc><changefreq>yearly</changefreq><priority>0.3</priority></url>
  <url><loc>{}/terms</loc><changefreq>yearly</changefreq><priority>0.3</priority></url>
  <url><loc>{}/docs/secrets</loc><changefreq>monthly</changefreq><priority>0.6</priority></url>
</urlset>"#,
        base, base, base, base, base
    );
    (
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        body,
    )
}

async fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn app(app_state: Arc<AppState>) -> Router {
    let secret_router = Router::new()
        .route("/secret", post(create_secret))
        .route("/secret/:shared_secret_id", get(get_secret_by_id))
        .route("/", get(views::index::index))
        .route("/get_secret", get(views::get_secret::get_secret))
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/privacy", get(|| async { PrivacyHtml }))
        .route("/terms", get(|| async { TermsHtml }))
        .route("/integrations", get(|| async { IntegrationsHtml }))
        .route("/docs/secrets", get(|| async { DocsSecretsHtml }))
        .route(
            "/contact",
            get(|| async {
                axum::response::Redirect::permanent("mailto:whisper@quentinvedrenne.com")
            }),
        )
        .route("/assets/*path", get(serve_asset))
        .route("/robots.txt", get(robots_txt))
        .route("/sitemap.xml", get(sitemap_xml))
        .with_state(app_state.clone());

    let slack_router = Router::new()
        .route(
            "/slack/whisper",
            post(slack::whisper_command::handle_whisper),
        )
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            slack::signature::verify_slack_signature,
        ))
        .with_state(app_state.clone());

    let managed_secret_router = Router::new()
        .route(
            "/v1/secrets/:id",
            put(managed_secret_api::upsert_secret::upsert_secret)
                .get(managed_secret_api::get_secret::get_secret)
                .delete(managed_secret_api::delete_secret::delete_secret),
        )
        .with_state(app_state.clone());

    Router::new()
        .merge(secret_router)
        .merge(slack_router)
        .merge(managed_secret_router)
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=63072000; includeSubDomains"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            axum::http::header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(
                "default-src 'self'; base-uri 'self'; object-src 'none'; frame-ancestors 'none'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com https://api.fontshare.com; font-src 'self' https://fonts.gstatic.com https://cdn.fontshare.com; script-src 'self' 'unsafe-inline' https://www.googletagmanager.com https://cdn.mxpnl.com; connect-src 'self' https://api-eu.mixpanel.com https://cdn.mxpnl.com https://www.google-analytics.com https://*.google-analytics.com https://*.analytics.google.com https://*.googletagmanager.com; img-src 'self' data: https://www.googletagmanager.com",
            ),
        ))
}
