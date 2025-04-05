//! Contains tracing and opentelemetry initializer.
//!
//! # Example
//!
//! ```rust,no_run
//! use caslex_extra::observability::{setup_opentelemetry, unset_opentelemetry};
//! use tracing::{Level, span};
//!
//! setup_opentelemetry("my_service_name");
//!
//! span!(Level::INFO, "my_span", answer = 42);
//!
//! unset_opentelemetry("my_service_name");
//! ```
//!
//! Log level of logs and traces configure via `LOG_LEVEL` and `OTEL_LOG_LEVEL` environment
//! variables.

use std::{env, sync::OnceLock};

use opentelemetry::{KeyValue, global, trace::TracerProvider};
use opentelemetry_otlp::{Protocol, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    propagation::TraceContextPropagator,
    trace::{Sampler, SdkTracerProvider},
};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan, prelude::*};

use crate::closer;

const DEFAULT_LOG_LEVEL: &str = "debug";

fn get_resource(name: String) -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();
    RESOURCE
        .get_or_init(|| {
            Resource::builder()
                .with_service_name(name)
                .with_attribute(KeyValue::new("service.version", env!("CARGO_PKG_VERSION")))
                .build()
        })
        .clone()
}

fn get_tracer_provider(name: String) -> SdkTracerProvider {
    static INSTANCE: OnceLock<SdkTracerProvider> = OnceLock::new();
    INSTANCE.get_or_init(|| init_traces(name)).clone()
}

fn init_traces(name: String) -> SdkTracerProvider {
    const DEFAULT_SAMPLE_RATIO: f64 = 1.0;
    let exporter = SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .build()
        .expect("Failed to create span exporter");

    let ratio = env::var("OTEL_SAMPLING_RATIO")
        .unwrap_or_else(|_| DEFAULT_SAMPLE_RATIO.to_string())
        .parse::<f64>()
        .expect("Invalid OTEL_SAMPLING_RATIO");

    let sampler = if ratio < DEFAULT_SAMPLE_RATIO {
        Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(ratio)))
    } else {
        Sampler::AlwaysOn
    };

    SdkTracerProvider::builder()
        .with_resource(get_resource(name))
        .with_batch_exporter(exporter)
        .with_sampler(sampler)
        .build()
}

/// Setup opentelemetry.
///
/// Init opentelemetry tracer provider and tracing.
pub fn setup_opentelemetry(name: &'static str) -> SdkTracerProvider {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer_provider = get_tracer_provider(name.to_owned());
    // Set the global tracer provider using a clone of the tracer_provider.
    // Setting global tracer provider is required if other parts of the application
    // uses global::tracer() or global::tracer_with_version() to get a tracer.
    // Cloning simply creates a new reference to the same tracer provider. It is
    // important to hold on to the tracer_provider here, so as to invoke
    // shutdown on it when application ends.
    global::set_tracer_provider(tracer_provider.clone());

    // Create a new opentelemetry layer
    let otel_layer =
        tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer(name.to_owned()));

    // For the OpenTelemetry layer, add a tracing filter to filter events from
    // OpenTelemetry and its dependent crates (opentelemetry-otlp uses crates
    // like reqwest/tonic etc.) from being sent back to OTel itself, thus
    // preventing infinite telemetry generation. The filter levels are set as
    // follows:
    // - Allow `info` level and above by default.
    // - Restrict `opentelemetry`, `hyper`, `tonic`, and `reqwest` completely.
    // Note: This will also drop events from crates like `tonic` etc. even when
    // they are used outside the OTLP Exporter. For more details, see:
    // https://github.com/open-telemetry/opentelemetry-rust/issues/761
    let otel_log_level =
        env::var("OTEL_LOG_LEVEL").unwrap_or_else(|_| DEFAULT_LOG_LEVEL.to_string());

    let filter_otel = EnvFilter::new(otel_log_level.clone())
        .add_directive(
            format!("{}={}", name.to_owned(), otel_log_level)
                .parse()
                .unwrap(),
        )
        .add_directive("axum=off".parse().unwrap())
        .add_directive("hyper=off".parse().unwrap())
        .add_directive("opentelemetry=off".parse().unwrap())
        .add_directive("h2=off".parse().unwrap())
        .add_directive("reqwest=off".parse().unwrap());
    let otel_layer = otel_layer.with_filter(filter_otel);

    // Create a new tracing::Fmt layer to print the logs to stdout. It has a
    // default filter of `info` level and above, and `debug` and above for logs
    // from OpenTelemetry crates. The filter levels can be customized as needed.
    let fmt_log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| DEFAULT_LOG_LEVEL.to_string());

    let filter_fmt = EnvFilter::new(fmt_log_level.clone())
        .add_directive(
            format!("{}={}", name.to_owned(), fmt_log_level)
                .parse()
                .unwrap(),
        )
        .add_directive("hyper=error".parse().unwrap())
        .add_directive("h2=error".parse().unwrap())
        .add_directive("reqwest=error".parse().unwrap())
        .add_directive("tower_http=error".parse().unwrap())
        .add_directive("axum::rejection=trace".parse().unwrap())
        .add_directive("tokio_postgres=error".parse().unwrap())
        .add_directive("tracing=error".parse().unwrap())
        .add_directive("opentelemetry=error".parse().unwrap());
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_span_events(FmtSpan::CLOSE)
        .with_thread_names(true)
        .json()
        .flatten_event(true)
        .with_level(true)
        .with_line_number(true)
        .with_filter(filter_fmt);

    // Initialize the tracing subscriber with the OpenTelemetry layer and the
    // Fmt layer.
    tracing_subscriber::registry()
        .with(otel_layer)
        .with(fmt_layer)
        .init();

    // Add callback to unset opentelemetry automatically
    closer::push_callback(Box::new(|| unset_opentelemetry(name)));

    tracer_provider
}

/// Close tracer provider.
pub fn unset_opentelemetry(name: &str) {
    if let Err(e) = get_tracer_provider(name.to_owned()).shutdown() {
        tracing::error!("Failed to shutdown tracer provider: {}", e);
    };
}
