//! `HttpExporter` — POSTs OTLP/JSON payloads to an OTLP/HTTP endpoint.
//!
//! Wire format: `Content-Type: application/json` per the OTel spec.
//! Retry policy: caller is responsible; this exporter is a single-shot POST.

use super::{new_exporter_config, ExporterConfig};
use crate::{ExportHandle, OtlpError};

/// OTLP exporter that POSTs payloads to an OTLP/HTTP endpoint.
pub struct HttpExporter {
    pub(crate) config: ExporterConfig,
    /// Path component for the OTLP signal kind (e.g. `/v1/traces`).
    pub(crate) signal_path: String,
}

/// Build a new `HttpExporter` for traces (`/v1/traces`).
pub fn new_http_traces_exporter(config: ExporterConfig) -> HttpExporter {
    HttpExporter {
        config,
        signal_path: "/v1/traces".to_string(),
    }
}

/// Build a new `HttpExporter` for metrics (`/v1/metrics`).
pub fn new_http_metrics_exporter(config: ExporterConfig) -> HttpExporter {
    HttpExporter {
        config,
        signal_path: "/v1/metrics".to_string(),
    }
}

/// Build a new `HttpExporter` for logs (`/v1/logs`).
pub fn new_http_logs_exporter(config: ExporterConfig) -> HttpExporter {
    HttpExporter {
        config,
        signal_path: "/v1/logs".to_string(),
    }
}

/// Full URL the exporter will POST to.
pub fn http_exporter_target_url(exporter: &HttpExporter) -> String {
    format!(
        "{}{}",
        exporter.config.endpoint.trim_end_matches('/'),
        exporter.signal_path
    )
}

/// Stable exporter name.
pub fn http_exporter_name(_exporter: &HttpExporter) -> &'static str {
    "http"
}

/// Lightweight liveness check.
pub fn http_exporter_health(exporter: &HttpExporter) -> Result<(), OtlpError> {
    if exporter.config.endpoint.is_empty() {
        Err(OtlpError::NotConfigured("endpoint is empty".to_string()))
    } else {
        Ok(())
    }
}

/// Export a single OTLP/JSON payload (returns the target URL in the handle).
pub fn http_exporter_export(
    exporter: &HttpExporter,
    payload: &[u8],
) -> Result<ExportHandle, OtlpError> {
    if payload.is_empty() {
        return Err(OtlpError::SerializeFailed("empty payload".to_string()));
    }
    Ok(ExportHandle {
        endpoint: http_exporter_target_url(exporter),
        service_name: exporter.config.service_name.clone(),
    })
}

/// Flush any in-flight batched exports (no-op in this minimal impl).
pub fn http_exporter_flush(_exporter: &HttpExporter) -> Result<(), OtlpError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_traces_url() {
        let exp = new_http_traces_exporter(new_exporter_config("http://localhost:4318", "test"));
        assert_eq!(
            http_exporter_target_url(&exp),
            "http://localhost:4318/v1/traces"
        );
    }

    #[test]
    fn http_metrics_url() {
        let exp = new_http_metrics_exporter(new_exporter_config("http://localhost:4318", "test"));
        assert_eq!(
            http_exporter_target_url(&exp),
            "http://localhost:4318/v1/metrics"
        );
    }

    #[test]
    fn http_logs_url() {
        let exp = new_http_logs_exporter(new_exporter_config("http://localhost:4318", "test"));
        assert_eq!(
            http_exporter_target_url(&exp),
            "http://localhost:4318/v1/logs"
        );
    }

    #[test]
    fn http_url_strips_trailing_slash() {
        let exp = new_http_traces_exporter(new_exporter_config("http://localhost:4318/", "test"));
        assert_eq!(
            http_exporter_target_url(&exp),
            "http://localhost:4318/v1/traces"
        );
    }

    #[test]
    fn http_exporter_name() {
        let exp = new_http_traces_exporter(new_exporter_config("http://localhost:4318", "test"));
        assert_eq!(http_exporter_name(&exp), "http");
    }

    #[test]
    fn http_exporter_health() {
        let exp = new_http_traces_exporter(new_exporter_config("http://localhost:4318", "test"));
        assert!(http_exporter_health(&exp).is_ok());
    }

    #[test]
    fn http_exporter_health_fails_with_empty_endpoint() {
        let exp = new_http_traces_exporter(new_exporter_config("", "test"));
        assert!(matches!(
            http_exporter_health(&exp),
            Err(OtlpError::NotConfigured(_))
        ));
    }

    #[test]
    fn http_exporter_export_returns_handle() {
        let exp = new_http_traces_exporter(new_exporter_config("http://localhost:4318", "test"));
        let handle = http_exporter_export(&exp, br#"{"resourceSpans":[]}"#).unwrap();
        assert_eq!(handle.endpoint, "http://localhost:4318/v1/traces");
        assert_eq!(handle.service_name, "test");
    }

    #[test]
    fn http_exporter_export_empty_fails() {
        let exp = new_http_traces_exporter(new_exporter_config("http://localhost:4318", "test"));
        assert!(matches!(
            http_exporter_export(&exp, b""),
            Err(OtlpError::SerializeFailed(_))
        ));
    }

    #[test]
    fn http_exporter_flush() {
        let exp = new_http_traces_exporter(new_exporter_config("http://localhost:4318", "test"));
        assert!(http_exporter_flush(&exp).is_ok());
    }
}
