use askama_axum::Template;

use super::seo::SeoMeta;

#[derive(Template)]
#[template(path = "privacy.html")]
pub struct PrivacyHtml {
    pub seo: SeoMeta,
}

#[derive(Template)]
#[template(path = "terms.html")]
pub struct TermsHtml {
    pub seo: SeoMeta,
}
