use tracing_subscriber::fmt::format::FmtSpan;

pub fn init() {
    // Use pretty logging in development, JSON in production
    let env = std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());

    if env == "production" {
        // JSON format for production (machine-readable, no colors)
        tracing_subscriber::fmt()
            .json()
            .with_ansi(false)
            .with_current_span(false)
            .with_target(false)
            .init();
    } else {
        // Pretty format for development (human-readable, with colors)
        tracing_subscriber::fmt()
            .with_target(false)
            .with_thread_ids(false)
            .with_span_events(FmtSpan::NONE)
            .init();
    }
}
