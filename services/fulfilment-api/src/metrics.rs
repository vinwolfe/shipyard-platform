//! Prometheus metrics registry for fulfilment-api.
//!
//! Contract:
//! - `/metrics` exposes Prometheus text format
//! - HTTP metrics:
//!   - http_requests_total{method,route,status}
//!   - http_request_duration_seconds_bucket{method,route,status,le}
//!
//! Notes:
//! - `/metrics` is excluded from HTTP metrics to avoid scrape noise.
//! - Route label uses Axum matched route (e.g. `/api/v1/orders/validate`) to avoid high cardinality.

use once_cell::sync::Lazy;
use prometheus_client::{
    encoding::{EncodeLabelSet, text::encode},
    metrics::{
        counter::Counter,
        family::Family,
        histogram::{Histogram, exponential_buckets},
    },
    registry::Registry,
};
use std::{sync::Mutex, time::Duration};

pub const PROM_CONTENT_TYPE: &str = "text/plain; version=0.0.4; charset=utf-8";

pub static METRICS: Lazy<Metrics> = Lazy::new(Metrics::new);

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct HttpLabels {
    pub method: String,
    pub route: String,
    pub status: String,
}

pub struct Metrics {
    registry: Mutex<Registry>,
    http_requests_total: Family<HttpLabels, Counter<u64>>,
    http_request_duration_seconds: Family<HttpLabels, Histogram>,
}

impl Metrics {
    fn new() -> Self {
        let mut registry = Registry::default();

        let http_requests_total: Family<HttpLabels, Counter<u64>> = Family::default();
        let http_request_duration_seconds: Family<HttpLabels, Histogram> =
            Family::new_with_constructor(|| {
                // 5ms start, x2 growth, 12 buckets (~10s upper range)
                Histogram::new(exponential_buckets(0.005, 2.0, 12))
            });

        registry.register(
            "http_requests",
            "Total HTTP requests (excluding /metrics).",
            http_requests_total.clone(),
        );
        registry.register(
            "http_request_duration_seconds",
            "HTTP request duration in seconds (excluding /metrics).",
            http_request_duration_seconds.clone(),
        );

        Self {
            registry: Mutex::new(registry),
            http_requests_total,
            http_request_duration_seconds,
        }
    }

    pub fn encode(&self) -> String {
        let registry = self.registry.lock().expect("metrics registry poisoned");
        let mut out = String::new();
        encode(&mut out, &registry).expect("failed to encode metrics");
        out
    }

    pub fn record_http_request(&self, method: &str, route: &str, status: u16, duration: Duration) {
        let labels = HttpLabels {
            method: method.to_string(),
            route: route.to_string(),
            status: status.to_string(),
        };

        self.http_requests_total.get_or_create(&labels).inc();
        self.http_request_duration_seconds
            .get_or_create(&labels)
            .observe(duration.as_secs_f64());
    }
}
