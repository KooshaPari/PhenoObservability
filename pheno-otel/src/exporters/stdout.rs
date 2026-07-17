//! `StdoutExporter` — writes OTLP/JSON payloads to stderr.
//!
//! Useful for local dev, CI smoke tests, and dogfooding. **Not** for prod.

use super::{new_exporter_config, ExporterConfig};
use crate::{ExportHandle, OtlpError};

/// OTLP exporter that writes payloads to stderr.
pub struct StdoutExporter {
    pub(crate) config: ExporterConfig,
}

/// Build a new `StdoutExporter` with the given config.
pub fn new_stdout_exporter(config: ExporterConfig) -> StdoutExporter {
    StdoutExporter { config }
}

/// Stable exporter name.
pub fn stdout_exporter_name(_exporter: &StdoutExporter) -> &'static str {
    "stdout"
}

/// Lightweight liveness check.
pub fn stdout_exporter_health(exporter: &StdoutExporter) -> Result<(), OtlpError> {
    if exporter.config.endpoint.is_empty() {
        Err(OtlpError::NotConfigured("endpoint is empty".to_string()))
    } else {
        Ok(())
    }
}

/// Export a single OTLP/JSON payload to stderr.
pub fn stdout_exporter_export(
    exporter: &StdoutExporter,
    payload: &[u8],
) -> Result<ExportHandle, OtlpError> {
    if payload.is_empty() {
        return Err(OtlpError::SerializeFailed("empty payload".to_string()));
    }
    eprintln!(
        "[pheno-otel/stdout] endpoint={} service={} bytes={}",
        exporter.config.endpoint,
        exporter.config.service_name,
        payload.len()
    );
    Ok(ExportHandle {
        endpoint: exporter.config.endpoint.clone(),
        service_name: exporter.config.service_name.clone(),
    })
}

/// Flush any in-flight batched exports (no-op for stderr).
pub fn stdout_exporter_flush(_exporter: &StdoutExporter) -> Result<(), OtlpError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdout_exporter_name() {
        let exp = new_stdout_exporter(new_exporter_config("http://localhost:4318", "test"));
        assert_eq!(stdout_exporter_name(&exp), "stdout");
    }

    #[test]
    fn stdout_exporter_health() {
        let exp = new_stdout_exporter(new_exporter_config("http://localhost:4318", "test"));
        assert!(stdout_exporter_health(&exp).is_ok());
    }

    #[test]
    fn stdout_exporter_health_fails_with_empty_endpoint() {
        let exp = new_stdout_exporter(new_exporter_config("", "test"));
        assert!(matches!(
            stdout_exporter_health(&exp),
            Err(OtlpError::NotConfigured(_))
        ));
    }

    #[test]
    fn stdout_exporter_export_returns_handle() {
        let exp = new_stdout_exporter(new_exporter_config("http://localhost:4318", "test"));
        let handle = stdout_exporter_export(&exp, br#"{"resourceSpans":[]}"#).unwrap();
        assert_eq!(handle.endpoint, "http://localhost:4318");
        assert_eq!(handle.service_name, "test");
    }

    #[test]
    fn stdout_exporter_export_empty_fails() {
        let exp = new_stdout_exporter(new_exporter_config("http://localhost:4318", "test"));
        assert!(matches!(
            stdout_exporter_export(&exp, b""),
            Err(OtlpError::SerializeFailed(_))
        ));
    }

    #[test]
    fn stdout_exporter_flush() {
        let exp = new_stdout_exporter(new_exporter_config("http://localhost:4318", "test"));
        assert!(stdout_exporter_flush(&exp).is_ok());
    }
}
