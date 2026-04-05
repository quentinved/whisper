use askama_axum::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexHtml {
    pub shared_secret_id: Option<String>,
    pub error_shared_secret: Option<String>,
}

impl IndexHtml {
    pub fn new(shared_secret_id: Option<String>, error_shared_secret: Option<String>) -> Self {
        Self {
            shared_secret_id,
            error_shared_secret,
        }
    }
}
