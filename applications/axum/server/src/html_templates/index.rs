use askama_axum::Template;

use super::seo::SeoMeta;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexHtml {
    pub seo: SeoMeta,
    pub shared_secret_id: Option<String>,
    pub error_shared_secret: Option<String>,
}

impl IndexHtml {
    pub fn new(
        seo: SeoMeta,
        shared_secret_id: Option<String>,
        error_shared_secret: Option<String>,
    ) -> Self {
        Self {
            seo,
            shared_secret_id,
            error_shared_secret,
        }
    }
}
