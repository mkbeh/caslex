use std::{fmt::Display, time::Duration};

use axum::{Router, body::HttpBody, extract::MatchedPath};
use axum_core::body::Body;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::{Level, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::extractors;

pub fn with_trace_layer(router: Router) -> Router {
    router.layer(
        TraceLayer::new_for_http()
            .make_span_with(make_span_with_handler)
            .on_request(())
            .on_body_chunk(())
            .on_eos(())
            .on_response(on_response_handler)
            .on_failure(on_failure_handler),
    )
}

enum OtelStatusCode {
    Ok,
    Error,
}

impl Display for OtelStatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            OtelStatusCode::Ok => write!(f, "OK"),
            OtelStatusCode::Error => write!(f, "ERROR"),
        }
    }
}

fn make_span_with_handler(request: &axum_core::extract::Request<Body>) -> Span {
    let matched_path = request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str);

    tracing::span!(
        Level::TRACE,
        "http_request",
        otel.kind = "server",
        otel.status_code = tracing::field::Empty,
        otel.status_message = tracing::field::Empty,
        http.method = ?request.method(),
        http.path = matched_path,
        http.query_params = request.uri().query(),
        http.status_code = tracing::field::Empty,
        http.request_size = request.body().size_hint().lower(),
        http.response_size = tracing::field::Empty,
        user_agent = extractors::user_agent(request),
        http.request_headers = ?request.headers(),
    )
}

fn on_response_handler(
    response: &axum_core::response::Response<Body>,
    _latency: Duration,
    span: &Span,
) {
    span.record(
        "http.status_code",
        tracing::field::display(response.status()),
    );
    span.record(
        "http.response_size",
        tracing::field::display(response.body().size_hint().lower()),
    );

    match response.status().as_u16() {
        0..=399 => {
            span_ok(span);
        }
        _ => {
            span_err(span, "received error response".to_owned());
        }
    }
}

fn on_failure_handler(error: ServerErrorsFailureClass, _latency: Duration, span: &Span) {
    span_err(span, error.to_string());
}

fn span_ok(span: &Span) {
    span.set_attribute("otel.status_code", OtelStatusCode::Ok.to_string());
}

fn span_err(span: &Span, message: String) {
    span.set_attribute("otel.status_code", OtelStatusCode::Error.to_string());
    span.set_attribute("otel.status_message", message);
}
