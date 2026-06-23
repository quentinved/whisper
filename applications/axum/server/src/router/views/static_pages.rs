use std::sync::Arc;

use axum::extract::State;

use crate::app_state::AppState;
use crate::html_templates::{
    docs::DocsSecretsHtml,
    integrations::IntegrationsHtml,
    legal::{PrivacyHtml, TermsHtml},
    seo::SeoMeta,
};

pub async fn integrations(State(app_state): State<Arc<AppState>>) -> IntegrationsHtml {
    let seo = SeoMeta::new(
        app_state.url(),
        "/integrations",
        "Integrations — Whisper",
        "Integrate Whisper with Slack, Discord, Raycast and Microsoft Teams. Share encrypted, self-destructing secrets directly from your favorite tools.",
    );
    IntegrationsHtml { seo }
}

pub async fn docs_secrets(State(app_state): State<Arc<AppState>>) -> DocsSecretsHtml {
    let seo = SeoMeta::new(
        app_state.url(),
        "/docs/secrets",
        "whisper-secrets CLI Docs — Whisper",
        "Documentation for whisper-secrets: a zero-knowledge .env secret manager. Push and pull encrypted secrets from the command line. No signup required.",
    );
    DocsSecretsHtml { seo }
}

pub async fn privacy(State(app_state): State<Arc<AppState>>) -> PrivacyHtml {
    let seo = SeoMeta::new(
        app_state.url(),
        "/privacy",
        "Privacy Policy — Whisper",
        "Whisper's privacy policy. We collect minimal data — your secrets are end-to-end encrypted and automatically deleted after expiration or first retrieval.",
    );
    PrivacyHtml { seo }
}

pub async fn terms(State(app_state): State<Arc<AppState>>) -> TermsHtml {
    let seo = SeoMeta::new(
        app_state.url(),
        "/terms",
        "Terms of Service — Whisper",
        "Whisper's terms of service. Read about acceptable use, data handling, and the rules for using this encrypted secret-sharing service.",
    );
    TermsHtml { seo }
}
