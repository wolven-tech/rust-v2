//! The `apps/api` binary. The only server process in rust-v2.

use std::{net::SocketAddr, sync::Arc};

use api::{
    build_router,
    infrastructure::{jobs, observability, state::build_state},
};
use rv2_shared::ServerConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // First, so every failure below is itself logged.
    let observability = observability::init();

    // Before the router, because the recorder has to be installed before the
    // first `metrics!` macro runs — and after logging, so its own decisions are
    // visible. A bad `METRICS_ADDR` fails the boot on purpose: a metrics
    // endpoint that silently is not there is how a dashboard ends up showing a
    // flat line everyone reads as "no traffic".
    observability::init_metrics()?;

    // R6: this fails loudly and by name if ALLSOURCE_CORE_URL /
    // ALLSOURCE_QUERY_URL are unset. There is deliberately no default port.
    let config = ServerConfig::from_env()?;
    tracing::info!(?config, "starting rust-v2 api");

    let state = Arc::new(build_state(&config).await?);
    let app = build_router(Arc::clone(&state));

    // Started before the listener so the first scrape after a boot has real
    // values rather than an absent series, which reads on a graph as a gap
    // rather than as "just started".
    let jobs = jobs::start(Arc::clone(&state));

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

    // Everything below runs AFTER serve returns, in dependency order.

    // Jobs first: they touch state, and one mid-run when the process exits is
    // the kind of half-finished work that gets discovered months later.
    jobs.shutdown().await;

    // Then analytics. `track` is fire-and-forget (see that crate's module
    // docs), so at any instant some events exist only in an in-process queue.
    // Exiting without this drops them, and drops them *silently* — the symptom
    // is "PostHog is missing the last few minutes before every deploy", which
    // nobody traces back to a missing flush.
    state.analytics.shutdown().await;
    tracing::info!("shutdown complete");

    // Last: the exporter batches, so the spans describing this shutdown exist
    // only in memory until it drains.
    observability.shutdown();

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
