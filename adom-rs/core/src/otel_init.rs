use crate::config::Config;
use crate::config::types::OtelExporterKind as Kind;
use crate::config::types::OtelHttpProtocol as Protocol;
use crate::default_client::originator;
use adom_otel::config::OtelExporter;
use adom_otel::config::OtelHttpProtocol;
use adom_otel::config::OtelSettings;
use adom_otel::otel_provider::OtelProvider;
use std::error::Error;

/// Build an OpenTelemetry provider from the app Config.
///
/// Returns `None` when OTEL export is disabled.
pub fn build_provider(
    config: &Config,
    service_version: &str,
) -> Result<Option<OtelProvider>, Box<dyn Error>> {
    let exporter = match &config.otel.exporter {
        Kind::None => OtelExporter::None,
        Kind::OtlpHttp {
            endpoint,
            headers,
            protocol,
        } => {
            let protocol = match protocol {
                Protocol::Json => OtelHttpProtocol::Json,
                Protocol::Binary => OtelHttpProtocol::Binary,
            };

            OtelExporter::OtlpHttp {
                endpoint: endpoint.clone(),
                headers: headers
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
                protocol,
            }
        }
        Kind::OtlpGrpc { endpoint, headers } => OtelExporter::OtlpGrpc {
            endpoint: endpoint.clone(),
            headers: headers
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        },
    };

    OtelProvider::from(&OtelSettings {
        service_name: originator().value.to_owned(),
        service_version: service_version.to_string(),
        adom_home: config.adom_home.clone(),
        environment: config.otel.environment.to_string(),
        exporter,
    })
}

/// Filter predicate for exporting only Adom-owned events via OTEL.
/// Keeps events that originated from adom_otel module
pub fn adom_export_filter(meta: &tracing::Metadata<'_>) -> bool {
    meta.target().starts_with("adom_otel")
}
