use askama_axum::Template;

#[derive(Template)]
#[template(path = "get_secret.html")]
pub struct GetSecretHtml {
    pub shared_secret: Option<String>,
    pub self_destruct: bool,
    pub error_shared_secret: Option<String>,
}

impl GetSecretHtml {
    pub fn new(
        shared_secret: Option<String>,
        self_destruct: Option<bool>,
        error_shared_secret: Option<String>,
    ) -> Self {
        Self {
            shared_secret,
            self_destruct: self_destruct.unwrap_or(false),
            error_shared_secret,
        }
    }
}
