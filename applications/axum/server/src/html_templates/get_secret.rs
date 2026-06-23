use askama_axum::Template;

use super::seo::SeoMeta;

#[derive(Template)]
#[template(path = "get_secret.html")]
pub struct GetSecretHtml {
    pub seo: SeoMeta,
    pub error_shared_secret: Option<String>,
}

impl GetSecretHtml {
    pub fn new(seo: SeoMeta, error_shared_secret: Option<String>) -> Self {
        Self {
            seo,
            error_shared_secret,
        }
    }
}
