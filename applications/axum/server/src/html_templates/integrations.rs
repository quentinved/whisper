use askama_axum::Template;

#[derive(Template)]
#[template(path = "integrations.html")]
pub struct IntegrationsHtml;
