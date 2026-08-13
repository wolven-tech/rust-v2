//! Logs, metrics and traces.
//!
//! ## Everything here is opt-in, and that is the design
//!
//! | Signal | Off unless | When off |
//! |---|---|---|
//! | Structured logs | `LOG_FORMAT=json` | Human-readable to stdout |
//! | Metrics | `METRICS_ADDR` is set | No recorder is installed |
//! | Traces | `OTEL_EXPORTER_OTLP_ENDPOINT` is set | No exporter, no batching task |
//!
//! A fresh clone runs with none of them, needs no collector, and pays for none
//! of it. `metrics` is a facade: with no recorder installed its macros compile
//! to approximately nothing, so the instrumentation in `rv2-jobs` and in the
//! HTTP layer costs nothing when metrics are off rather than being conditionally
//! compiled out.
//!
//! ## Why the metrics listener is a separate port
//!
//! `/metrics` is not mounted on the application router. Scrape output names
//! every route, reports request volumes, and is a free reconnaissance surface;
//! it also has no business being subject to — or exempt from — the application's
//! rate limiter and CORS policy. A separate listener that is simply never
//! published outside the cluster is a far cruder control than an auth check, and
//! a much harder one to get subtly wrong.
//!
//! ## Cardinality is the thing that breaks metrics
//!
//! HTTP metrics are labelled with the **matched route pattern** (`/posts/{id}`),
//! never the request path (`/posts/4862f98b-…`). Labelling by path gives one
//! time series per uuid; a few thousand posts is a few thousand series that
//! never expire, and the Prometheus instance falls over long before anyone
//! connects it to this line of code.

use std::net::SocketAddr;
use std::time::Instant;

use axum::{extract::MatchedPath, http::Request, middleware::Next, response::Response};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig as _;
use opentelemetry_sdk::{Resource, trace::SdkTracerProvider};
use tracing_subscriber::{
    EnvFilter, Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _,
};

/// Handles that must outlive `main`'s body so the signals can be drained on the
/// way out.
///
/// `Option` because traces are opt-in, so [`Self::shutdown`] needs no branch at
/// the call site. Metrics need no handle: the recorder is process-global and
/// scraped rather than pushed, so there is nothing to flush.
#[derive(Default)]
pub struct Observability {
    traces: Option<SdkTracerProvider>,
}

impl Observability {
    /// Flush anything still buffered.
    ///
    /// The OTLP exporter batches, so spans from the last few seconds of a
    /// process exist only in memory until this runs. Skipping it loses exactly
    /// the spans most likely to explain why the process is shutting down.
    pub fn shutdown(&self) {
        if let Some(provider) = &self.traces
            && let Err(error) = provider.shutdown()
        {
            tracing::warn!(%error, "the trace exporter did not shut down cleanly");
        }
    }
}

/// Install logging, and traces if a collector endpoint is configured.
///
/// Called once, before anything else, so a failure during the rest of boot is
/// itself logged.
#[must_use]
pub fn init() -> Observability {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,api=debug"));

    // `LOG_FORMAT=json` emits one JSON object per event, so the fields this
    // codebase already attaches reach an aggregator as fields rather than as a
    // line to re-parse with a regex.
    //
    // Deliberately not auto-detected from a TTY: a container run interactively
    // must log the way it does in production, or the format silently differs
    // between the place you debug and the place it matters.
    let json = std::env::var("LOG_FORMAT").is_ok_and(|f| f.eq_ignore_ascii_case("json"));
    let fmt_layer = if json {
        tracing_subscriber::fmt::layer().json().boxed()
    } else {
        tracing_subscriber::fmt::layer().boxed()
    };

    let traces = build_tracer_provider();
    let otel_layer = traces
        .as_ref()
        .map(|provider| tracing_opentelemetry::layer().with_tracer(provider.tracer("api")));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .with(otel_layer)
        .init();

    if traces.is_some() {
        tracing::info!("exporting traces over OTLP");
    }
    Observability { traces }
}

/// The OTLP exporter, if `OTEL_EXPORTER_OTLP_ENDPOINT` names a collector.
///
/// `OTEL_*` are the names the OpenTelemetry specification defines, not names
/// invented here — an operator who has configured any other service already
/// knows them, and a sidecar collector often injects them automatically.
fn build_tracer_provider() -> Option<SdkTracerProvider> {
    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .ok()
        .filter(|e| !e.trim().is_empty())?;

    let exporter = match opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint(format!("{}/v1/traces", endpoint.trim_end_matches('/')))
        .build()
    {
        Ok(exporter) => exporter,
        Err(error) => {
            // A collector that will not build is a configuration mistake, not a
            // reason to refuse to serve traffic. Log it and run without traces —
            // the alternative is an observability misconfiguration causing the
            // outage it was meant to help diagnose.
            tracing::warn!(%error, %endpoint, "could not build the OTLP exporter; traces disabled");
            return None;
        }
    };

    Some(
        SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_resource(
                Resource::builder()
                    .with_service_name(env!("CARGO_PKG_NAME"))
                    .build(),
            )
            .build(),
    )
}

/// Install the Prometheus recorder and its scrape listener.
///
/// Returns `Ok(false)` when `METRICS_ADDR` is unset, which is not a failure:
/// no recorder is installed and every `metrics` macro in the workspace becomes
/// a no-op.
///
/// # Errors
///
/// A string describing an unparseable `METRICS_ADDR` or a listener that could
/// not bind. Both are configuration errors and both should fail the boot — a
/// metrics endpoint that silently is not there is how a dashboard ends up
/// showing a flat line that everyone reads as "no traffic".
pub fn init_metrics() -> Result<bool, String> {
    let Some(raw) = std::env::var("METRICS_ADDR")
        .ok()
        .filter(|a| !a.trim().is_empty())
    else {
        tracing::info!("METRICS_ADDR unset; metrics disabled");
        return Ok(false);
    };

    let addr: SocketAddr = raw
        .trim()
        .parse()
        .map_err(|e| format!("METRICS_ADDR `{raw}` is not a socket address: {e}"))?;

    metrics_exporter_prometheus::PrometheusBuilder::new()
        .with_http_listener(addr)
        .install()
        .map_err(|e| format!("could not start the metrics listener on {addr}: {e}"))?;

    tracing::info!(%addr, "metrics listener started");
    Ok(true)
}

/// Record one HTTP request.
///
/// Mounted below the rate limiter so that a 429 is counted: "how often are we
/// shedding load" is one of the few questions this layer exists to answer.
pub async fn track_requests(request: Request<axum::body::Body>, next: Next) -> Response {
    // The MATCHED ROUTE, not the path. See the module docs — this one line is
    // the difference between a handful of time series and one per entity id.
    // A request that matched no route reports `unmatched` rather than its raw
    // path, because 404-scanning traffic is exactly what would otherwise mint
    // unbounded labels.
    let route = request
        .extensions()
        .get::<MatchedPath>()
        .map_or_else(|| "unmatched".to_string(), |m| m.as_str().to_string());
    let method = request.method().as_str().to_string();

    let started = Instant::now();
    let response = next.run(request).await;
    let elapsed = started.elapsed();

    let status = response.status().as_u16().to_string();

    metrics::counter!(
        "http_requests_total",
        "method" => method.clone(),
        "route" => route.clone(),
        "status" => status,
    )
    .increment(1);

    metrics::histogram!(
        "http_request_duration_seconds",
        "method" => method,
        "route" => route,
    )
    .record(elapsed.as_secs_f64());

    response
}
