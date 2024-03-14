pub fn init() -> Result<(actix_web_opentelemetry::PrometheusMetricsHandler, opentelemetry_sdk::metrics::MeterProvider), opentelemetry::metrics::MetricsError> {
    let (metrics_handler, meter_provider) = {
        let registry = prometheus::Registry::new();
        let exporter = opentelemetry_prometheus::exporter()
            .with_registry(registry.clone())
            .build()?;
        let provider = opentelemetry_sdk::metrics::MeterProvider::builder()
            .with_reader(exporter)
            .with_resource(opentelemetry_sdk::resource::Resource::new([opentelemetry::KeyValue::new("service.name", "example")]))
            .with_view(
                opentelemetry_sdk::metrics::new_view(
                    opentelemetry_sdk::metrics::Instrument::new().name("http.server.duration"),
                        opentelemetry_sdk::metrics::Stream::new().aggregation(opentelemetry_sdk::metrics::Aggregation::ExplicitBucketHistogram {
                        boundaries: vec![
                            0.0, 0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1.0, 2.5,
                            5.0, 7.5, 10.0,
                        ],
                        record_min_max: true,
                    }),
                )
                .unwrap(),
            )
            .build();
        opentelemetry::global::set_meter_provider(provider.clone());

        (actix_web_opentelemetry::PrometheusMetricsHandler::new(registry), provider)
    };

    Ok((metrics_handler, meter_provider))
}