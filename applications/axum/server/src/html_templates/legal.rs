use askama_axum::Template;

#[derive(Template)]
#[template(path = "privacy.html")]
pub struct PrivacyHtml;

#[derive(Template)]
#[template(path = "terms.html")]
pub struct TermsHtml;
