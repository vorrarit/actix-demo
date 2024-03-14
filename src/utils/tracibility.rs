use tracing_subscriber::layer::SubscriberExt;
use opentelemetry_sdk::trace as sdktrace;

pub fn init(opentelemetry: Option<tracing_opentelemetry::OpenTelemetryLayer<tracing_subscriber::layer::Layered<tracing_subscriber::EnvFilter, tracing_subscriber::Registry>, sdktrace::Tracer>>) -> Result<(), Box<dyn std::error::Error>>  {
    tracing_log::LogTracer::init()?;

    if let Some(otel) = opentelemetry {
        let subscriber = tracing_subscriber::Registry::default()
            .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or(tracing_subscriber::EnvFilter::new("info")))
            .with(otel)
            .with(tracing_subscriber::fmt::layer().pretty());
        tracing::subscriber::set_global_default(subscriber)?;
    } else {
        let subscriber = tracing_subscriber::Registry::default()
            .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or(tracing_subscriber::EnvFilter::new("info")))
            .with(tracing_subscriber::fmt::layer().pretty());
        tracing::subscriber::set_global_default(subscriber)?;
    };

    Ok(())
}