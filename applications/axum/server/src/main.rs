use clap::Parser;
use std::{net::SocketAddr, sync::Arc};
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::info;
use whisper_core::commands::shared_secret::delete_expired_secrets::DeleteExpiredSecrets;
use whisper_server::aes_key::load_aes_key;
use whisper_server::{analytics, app_state, logger, options, postgresql, router};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize tracing
    logger::init();

    info!("Starting Whisper server...");

    // Get the options from the CLI
    let options = options::Options::parse();
    info!(
        "Configuration - Listen: {}:{}, Database: {}",
        options.listen_addr,
        options.port,
        options
            .url_postgresql
            .split('@')
            .next_back()
            .unwrap_or("hidden")
    );

    // Connect to the postgres database
    let pool = postgresql::create_db_pool(&options.url_postgresql).await.map_err(|e| {
        tracing::error!("Failed to connect to database: {:?}", e);
        e
    })?;
    info!("Connected to PostgreSQL database");

    whisper_postgresql::migrations::apply_migrations(&pool).await?;
    info!("Applied database migrations");

    // Load the key aesgcm from a file
    let secret_key = load_aes_key(&options.aes_key_path)?;
    info!("Loaded AES-256-GCM encryption key");

    // Set the context for our application
    let url = options
        .base_url
        .unwrap_or_else(|| format!("http://{}:{}", options.listen_addr, options.port));
    let slack_enabled = options.slack_signing_secret.is_some();

    let mut trackers: Vec<Arc<dyn analytics::AnalyticsTracker>> = vec![];
    if let Some(token) = options.mixpanel_token {
        info!("Mixpanel server-side tracking enabled");
        trackers.push(Arc::new(analytics::mixpanel::MixpanelTracker::new(token)));
    }
    if let (Some(measurement_id), Some(api_secret)) =
        (options.ga4_measurement_id, options.ga4_api_secret)
    {
        info!("GA4 server-side tracking enabled");
        trackers.push(Arc::new(analytics::ga4::GA4Tracker::new(
            measurement_id,
            api_secret,
        )));
    }
    let analytics: Arc<dyn analytics::AnalyticsTracker> =
        Arc::new(analytics::composite::CompositeTracker::new(trackers));

    let app_state = Arc::new(app_state::AppState::new(
        pool,
        secret_key,
        url,
        options.slack_signing_secret,
        analytics,
    ));
    if slack_enabled {
        info!("Slack slash command integration enabled");
    }
    let app_state_clone = Arc::clone(&app_state);

    // build our application with a route
    let router = router::app(app_state);

    // run our app with hyper, listening globally on port 3000
    // create tcp listener using the provided config
    let socket_addr = SocketAddr::new(options.listen_addr, options.port);
    let tcp_listener = tokio::net::TcpListener::bind(socket_addr).await?;
    let local_addr = tcp_listener.local_addr()?;
    info!("TCP listener bound to {}", local_addr);

    // Cron job for cleaning up expired secrets
    let sched: JobScheduler = JobScheduler::new().await?;

    // Define and add the cron job
    sched
        .add(Job::new_async("0 * * * * *", move |_, _| {
            let app_state = Arc::clone(&app_state_clone);
            Box::pin(async move {
                let repository = app_state.shared_secret_repository();

                let command = DeleteExpiredSecrets::new();
                match command.handle(&repository).await {
                    Ok(count) if count > 0 => {
                        tracing::info!("Cleanup job deleted {} expired secrets", count);
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("Failed to delete expired secrets: {:?}", e);
                    }
                }
            })
        })?)
        .await?;

    // Start the scheduler
    sched.start().await?;
    info!("Cron job scheduler started (cleanup every minute)");

    info!("");
    info!("========================================");
    info!("  Server running at http://{}", local_addr);
    info!("  Health check: http://{}/health", local_addr);
    info!("========================================");
    info!("");

    // run the server
    axum::serve(tcp_listener, router).await?;
    Ok(())
}
