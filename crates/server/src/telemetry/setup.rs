//! OpenTelemetry tracing: exports traces over OTLP/HTTP+JSON to the cluster collector
//! (`OTEL_TRACING_URL`, provided by the `traces` platform resource in app.yaml) when set,
//! and always keeps console logging. Mirrors home-gateway's setup.

use std::time::Duration;

use opentelemetry::{KeyValue, global, trace::TracerProvider};
use opentelemetry_otlp::{Protocol, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    propagation::TraceContextPropagator,
    trace::{BatchConfigBuilder, BatchSpanProcessor, SdkTracerProvider, Tracer},
};
use opentelemetry_semantic_conventions::resource::{
    DEPLOYMENT_ENVIRONMENT_NAME, K8S_NAMESPACE_NAME, K8S_POD_NAME, SERVICE_INSTANCE_ID,
    SERVICE_NAME, SERVICE_VERSION, TELEMETRY_SDK_LANGUAGE, TELEMETRY_SDK_NAME,
    TELEMETRY_SDK_VERSION,
};
use tracing::{Level, level_filters::LevelFilter};
use tracing_subscriber::{
    EnvFilter, Layer, filter::Targets, layer::SubscriberExt, util::SubscriberInitExt,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn telemetry_resource() -> Resource {
    let tags = vec![
        KeyValue::new(TELEMETRY_SDK_NAME, "otel-tracing-rs".to_string()),
        KeyValue::new(TELEMETRY_SDK_VERSION, VERSION.to_string()),
        KeyValue::new(TELEMETRY_SDK_LANGUAGE, "rust".to_string()),
        KeyValue::new(SERVICE_NAME, "spotify-server".to_string()),
        KeyValue::new(SERVICE_VERSION, VERSION.to_string()),
        KeyValue::new(
            DEPLOYMENT_ENVIRONMENT_NAME,
            if cfg!(debug_assertions) {
                "development"
            } else {
                "production"
            },
        ),
    ];

    let mut resource = Resource::builder_empty().with_attributes(tags);

    if let Ok(pod) = std::env::var("K8S_POD_NAME")
        && !pod.is_empty()
    {
        resource = resource.with_attributes([
            KeyValue::new(K8S_POD_NAME, pod.clone()),
            KeyValue::new(SERVICE_INSTANCE_ID, pod),
        ]);
    }

    if let Ok(namespace) = std::env::var("K8S_NAMESPACE_NAME")
        && !namespace.is_empty()
    {
        resource = resource.with_attributes([KeyValue::new(K8S_NAMESPACE_NAME, namespace)]);
    }

    resource.build()
}

fn external_tracer(ingest_url: String) -> Tracer {
    let span_exporter = opentelemetry_otlp::HttpExporterBuilder::default()
        .with_protocol(Protocol::HttpJson)
        .with_endpoint(ingest_url)
        .with_timeout(Duration::from_secs(3))
        .build_span_exporter()
        .expect("build the OTLP span exporter");

    let tracer_provider = SdkTracerProvider::builder()
        .with_span_processor(
            BatchSpanProcessor::builder(span_exporter)
                .with_batch_config(
                    BatchConfigBuilder::default()
                        .with_max_queue_size(20480)
                        .build(),
                )
                .build(),
        )
        .with_resource(telemetry_resource())
        .build();

    let tracer = tracer_provider.tracer("spotify-server");
    global::set_tracer_provider(tracer_provider);

    tracer
}

/// Spans handed to the exporter: perf noise from the exporter and SDK is kept off the wire
/// unless debugging, nothing else is elided. `sqlx::query` events ride along at DEBUG so db
/// work shows up under each span, dataloader plumbing is reduced to errors only.
fn exporter_filter(exporter_level: LevelFilter) -> Targets {
    Targets::default()
        .with_target("otel::tracing", Level::TRACE)
        .with_target("opentelemetry_sdk", exporter_level)
        .with_target("sqlx::query", Level::DEBUG)
        .with_target("async_graphql::dataloader", Level::WARN)
        .with_default(Level::INFO)
}

/// Console output, `RUST_LOG` (default `info`). Query events are always kept off the terminal,
/// as home-gateway does; the exporter level doesn't suppress what you asked for.
fn console_layer<S>() -> impl Layer<S>
where
    S: tracing::Subscriber + for<'span> tracing_subscriber::registry::LookupSpan<'span>,
{
    tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into())
                .add_directive("sqlx::query=off".parse().expect("static directive")),
        )
}

/// Installs the global subscriber. Call once, before anything logs. With `OTEL_TRACING_URL`
/// set, spans are exported in batches to the OTLP collector and the trace context
/// propagator is installed for outbound requests.
pub fn init() {
    let exporter_level = if cfg!(debug_assertions) {
        LevelFilter::OFF
    } else {
        LevelFilter::from_level(Level::INFO)
    };

    match std::env::var("OTEL_TRACING_URL") {
        Ok(ingest_url) if !ingest_url.is_empty() => {
            let tracer = external_tracer(ingest_url);

            opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

            tracing_subscriber::registry()
                .with(console_layer())
                .with(
                    tracing_opentelemetry::layer()
                        .with_tracer(tracer)
                        .with_filter(exporter_filter(exporter_level)),
                )
                .init();
        }
        _ => {
            tracing_subscriber::registry().with(console_layer()).init();
        }
    }
}
