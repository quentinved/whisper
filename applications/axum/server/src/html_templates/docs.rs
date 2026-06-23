use askama_axum::Template;

use super::seo::SeoMeta;

#[derive(Template)]
#[template(path = "docs_secrets.html")]
pub struct DocsSecretsHtml {
    pub seo: SeoMeta,
}
