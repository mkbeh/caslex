/// Provides prometheus plug-in metrics for Axum server.
///
/// This module tracks the following metrics under the following names:
///     - http_requests_total{method={"method"},path={"path"},status={"status"}}
///     - http_request_duration_sum{method={"method"},path={"path"},status={"status"}}
///     - http_request_duration_count{method={"method"},path={"path"},status={"status"}}
///     - http_request_duration_bucket{method={"method"},path={"path"},status={"status"},le={"
///       le"}}
///     - http_request_size{method={"method"},path={"path"},status={"status"}}
///     - http_response_size{method={"method"},path={"path"},status={"status"}}
use std::clone::Clone;

use axum::{
    body::{Bytes, HttpBody},
    extract::MatchedPath,
    middleware::Next,
};
use axum_core::{
    extract::Request,
    response::{IntoResponse, Response},
};
use http::header::CONTENT_TYPE;
use http_body_util::Full;
use lazy_static::lazy_static;
use prometheus::{
    CounterVec, Encoder, GaugeVec, HistogramVec, TextEncoder, register_counter_vec,
    register_gauge_vec, register_histogram_vec,
};
use tokio::time::Instant;

lazy_static! {
    static ref HTTP_COUNTER: CounterVec = register_counter_vec!(
        "http_requests_total",
        "Total number of HTTP requests made.",
        &["method", "path", "status"]
    )
    .unwrap();
    static ref HTTP_REQ_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "http_request_duration",
        "The HTTP request latencies in milliseconds.",
        &["method", "path", "status"]
    )
    .unwrap();
    static ref HTTP_REQ_BODY_GAUGE: GaugeVec = register_gauge_vec!(
        "http_request_size",
        "The metrics HTTP request sizes in bytes.",
        &["method", "path", "status"]
    )
    .unwrap();
    static ref HTTP_RESP_BODY_GAUGE: GaugeVec = register_gauge_vec!(
        "http_response_size",
        "The metrics HTTP request sizes in bytes.",
        &["method", "path", "status"]
    )
    .unwrap();
}

pub async fn metrics_handler(req: Request, next: Next) -> impl IntoResponse {
    let start = Instant::now();

    let path = match req.extensions().get::<MatchedPath>() {
        Some(matched_path) => matched_path.as_str().to_owned(),
        _ => req.uri().path().to_owned(),
    };

    let method = req.method().clone();
    let req_body_size = req.body().size_hint().lower();

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();
    let resp_body_size = response.body().size_hint().lower();

    let labels = &[method.as_str(), path.as_str(), status.as_str()];

    HTTP_COUNTER.with_label_values(labels).inc();
    HTTP_REQ_HISTOGRAM
        .with_label_values(labels)
        .observe(latency);
    HTTP_REQ_BODY_GAUGE
        .with_label_values(labels)
        .add(req_body_size as f64);
    HTTP_RESP_BODY_GAUGE
        .with_label_values(labels)
        .add(resp_body_size as f64);

    response
}

pub async fn prometheus_handler() -> impl IntoResponse {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    Response::builder()
        .status(200)
        .header(CONTENT_TYPE, encoder.format_type())
        .body(Full::new(Bytes::from(buffer)))
        .unwrap()
        .into_response()
}
