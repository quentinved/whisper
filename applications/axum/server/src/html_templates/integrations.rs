use askama_axum::Template;

use super::seo::SeoMeta;

#[derive(Template)]
#[template(path = "integrations.html")]
pub struct IntegrationsHtml {
    pub seo: SeoMeta,
}
