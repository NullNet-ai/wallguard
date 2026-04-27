use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry::trace::TracerProvider as _;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init(log_format: &str) -> Option<TracerProvider> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,wg_server=debug".into());

    let fmt_layer: Box<dyn tracing_subscriber::Layer<_> + Send + Sync> =
        if log_format == "json" {
            Box::new(tracing_subscriber::fmt::layer().json())
        } else {
            Box::new(tracing_subscriber::fmt::layer())
        };

    let otlp_endpoint = std::env::var("OTLP_ENDPOINT").ok();

    if let Some(endpoint) = otlp_endpoint {
        use opentelemetry_otlp::WithExportConfig;

        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .build()
            .expect("OTLP exporter init failed");

        let provider = TracerProvider::builder()
            .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
            .build();

        let otel_layer = tracing_opentelemetry::layer()
            .with_tracer(provider.tracer("wg-server"));

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .with(otel_layer)
            .init();

        Some(provider)
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();
        None
    }
}
