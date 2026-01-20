//! shipyard-observability
//!
//! Provides a golden-path observability initialisation:
//! - JSON structured logs to stdout (tracing-subscriber)
//! - OTLP trace export when endpoint is configured
//! - W3C trace context propagation
//!
//! Non-goals:
//! - Metrics exporter (handled separately)
//! - Vendor-specific logging backends

use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace as sdktrace;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Clone, Debug)]
pub struct ObservabilityConfig {
    /// Logical service name (shown in traces).
    pub service_name: String,

    /// OTLP endpoint (e.g. http://otelcol:4317). If None, traces are not exported.
    pub otlp_endpoint: Option<String>,

    /// Log filter string (RUST_LOG compatible).
    /// Example: "info,fulfilment_api=debug,shipyard_web=debug"
    pub log_filter: Option<String>,
}

pub fn init(cfg: ObservabilityConfig) {
    // W3C Trace Context propagation (traceparent/tracestate)
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Logs (always on): JSON to stdout
    let filter = cfg
        .log_filter
        .clone()
        .or_else(|| std::env::var("RUST_LOG").ok())
        .unwrap_or_else(|| "info".to_string());

    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(true);
    // .with_span_list(true); // drop for less noise

    let registry = tracing_subscriber::registry().with(tracing_subscriber::EnvFilter::new(filter));

    let warn_init = |err: tracing_subscriber::util::TryInitError| {
        tracing::warn!(error = %err, "observability already initialised or failed to initialise");
        // eprintln!("observability init skipped or failed: {err}");
    };

    // Traces: only wire exporter if endpoint is provided
    if let Some(endpoint) = cfg.otlp_endpoint {
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_trace_config(
                sdktrace::config().with_resource(opentelemetry_sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", cfg.service_name.clone()),
                ])),
            )
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(endpoint),
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .expect("failed to init OTLP pipeline");

        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        // Use try_init to avoid panics if init is called twice (tests / multiple binaries).
        if let Err(err) = registry.with(fmt_layer).with(otel_layer).try_init() {
            warn_init(err);
        }
    } else if let Err(err) = registry.with(fmt_layer).try_init() {
        warn_init(err);
    }
}

/// Shutdown hook to flush traces on exit (best-effort).
pub fn shutdown() {
    global::shutdown_tracer_provider();
}
