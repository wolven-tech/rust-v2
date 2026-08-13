//! The `apps/api` binary. The only server process in rust-v2.

use std::{net::SocketAddr, sync::Arc};

use api::{build_router, infrastructure::state::build_state};
use rv2_shared::ServerConfig;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

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

    // AFTER serve returns, so nothing tracked by an in-flight request is lost.
    //
    // `Analytics::track` is fire-and-forget (see that crate's module docs), so
    // at any instant some events exist only in an in-process queue. Exiting
    // without this drops them, and drops them *silently* — the symptom is
    // "PostHog is missing the last few minutes before every deploy", which
    // nobody traces back to a missing flush.
    state.analytics.shutdown().await;
    tracing::info!("shutdown complete");

    Ok(())
}

/// Structured logging, with the format chosen by the environment.
///
/// `LOG_FORMAT=json` emits one JSON object per event, which is what a log
/// aggregator needs in order to index the fields this codebase already attaches
/// (`%error`, `event = %event.name`, `worker = …`) as *fields* rather than as a
/// flat string it has to re-parse with a regex.
///
/// Deliberately not auto-detected from a TTY: a container that happens to be run
/// interactively must log the way it does in production, or the format silently
/// differs between the place you debug and the place it matters.
fn init_tracing() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,api=debug"));

    if std::env::var("LOG_FORMAT").is_ok_and(|f| f.eq_ignore_ascii_case("json")) {
        fmt().json().with_env_filter(filter).init();
    } else {
        fmt().with_env_filter(filter).init();
    }
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
