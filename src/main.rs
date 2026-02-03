use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod api;
mod auth;
mod config;
mod notify;
mod scheduler;

use api::{create_router, AppState};
use auth::AuthState;
use config::AppConfig;
use notify::Notifier;
use scheduler::NotificationScheduler;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Starting Agent Bark API...");

    // Load configuration
    let config = AppConfig::load()?;
    
    // Validate configuration
    if let Err(e) = config.validate() {
        tracing::error!("Configuration error: {}", e);
        return Err(e);
    }
    
    info!("Configuration loaded: server={}:{}, device_key=***", config.host, config.port);

    // Create notifier
    let notifier = Arc::new(Notifier::new(
        config.bark_url.clone(),
        config.device_key.clone(),
    ));

    // Create and start scheduler
    let scheduler = Arc::new(NotificationScheduler::new(Arc::clone(&notifier)).await?);
    scheduler.start().await?;

    // Create app state
    let state = AppState {
        notifier: Arc::clone(&notifier),
        scheduler: Arc::clone(&scheduler),
    };

    // Auth state
    let auth_state = AuthState {
        password: config.password.clone(),
    };

    // Create router with auth middleware
    let app = create_router(state, auth_state);

    // Start server
    let addr = config.socket_addr();
    info!("Server listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
