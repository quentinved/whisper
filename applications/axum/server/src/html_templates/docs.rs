use askama_axum::Template;

#[derive(Template)]
#[template(path = "docs_secrets.html")]
pub struct DocsSecretsHtml;
