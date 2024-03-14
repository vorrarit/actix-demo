use opentelemetry_sdk::trace as sdktrace;
use opentelemetry_otlp::WithExportConfig;

pub fn init_layer(grpc_url: &String, application_name: &String) -> Result<tracing_opentelemetry::OpenTelemetryLayer<tracing_subscriber::layer::Layered<tracing_subscriber::EnvFilter, tracing_subscriber::Registry>, sdktrace::Tracer>, opentelemetry::trace::TraceError> {
    opentelemetry::global::set_text_map_propagator(opentelemetry_sdk::propagation::TraceContextPropagator::new());

    // example from https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-otlp/examples/basic-otlp/src/main.rs
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(grpc_url)
                .with_protocol(opentelemetry_otlp::Protocol::Grpc),
        )
        .with_trace_config(
            sdktrace::config().with_resource(opentelemetry_sdk::Resource::new(vec![opentelemetry::KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                application_name.clone(),
            )])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    Ok(telemetry)
}