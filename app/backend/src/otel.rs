use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, metrics::SdkMeterProvider};

const DEFAULT_OTEL_SERVICE_NAME: &str = "backend";
const DEFAULT_OTEL_ENDPOINT: &str = "http://otel-lgtm:4317";

pub fn init_opentelemetry() -> Result<(), Box<dyn std::error::Error>> {
    let grpc_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| DEFAULT_OTEL_ENDPOINT.to_string());

    let service_name = std::env::var("OTEL_SERVICE_NAME")
        .unwrap_or_else(|_| DEFAULT_OTEL_SERVICE_NAME.to_string());
    let resource = Resource::builder()
        .with_attributes(vec![KeyValue::new("service.name", service_name)])
        .build();

    let metrics_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(&grpc_endpoint)
        .build()
        .expect("Failed to build metrics exporter");
    let metrics_reader = opentelemetry_sdk::metrics::PeriodicReader::builder(metrics_exporter)
        .with_interval(std::time::Duration::from_secs(10))
        .build();
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(metrics_reader)
        .with_resource(resource.clone())
        .build();
    global::set_meter_provider(meter_provider);

    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(grpc_endpoint)
        .build()
        .expect("Failed to create traces exporter");
    let tracer = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(span_exporter)
        .with_resource(resource)
        .build();
    global::set_tracer_provider(tracer);

    Ok(())
}
