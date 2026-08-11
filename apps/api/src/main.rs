//! The `apps/api` binary. The only server process in rust-v2.

use std::{net::SocketAddr, sync::Arc};

use api::{build_router, infrastructure::state::build_state};
use rv2_shared::ServerConfig;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,api=debug")),
        )
        .init();

    // R6: this fails loudly and by name if ALLSOURCE_CORE_URL /
    // ALLSOURCE_QUERY_URL are unset. There is deliberately no default port.
    let config = ServerConfig::from_env()?;
    tracing::info!(?config, "starting rust-v2 api");

    let state = Arc::new(build_state(&config).await?);
    let app = build_router(Arc::clone(&state));

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    tracing::info!(addr = %config.bind_addr, "listening");

    // `into_make_service_with_connect_info` is what puts `ConnectInfo` in the
    // request extensions, which the rate limiter needs to key by client ip.
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

/// Ctrl-C or SIGTERM. Without this, a container stop kills in-flight requests.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl-C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => tracing::info!("shutting down on Ctrl-C"),
        () = terminate => tracing::info!("shutting down on SIGTERM"),
    }
}
