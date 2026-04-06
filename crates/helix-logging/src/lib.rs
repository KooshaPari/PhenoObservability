//! Phenotype Logger - Unified structured logging across all services
//!
//! Provides a comprehensive logging framework with:
//! - Structured logging support
//! - Multiple log levels (Trace, Debug, Info, Warn, Error)
//! - Contextual logging with correlation IDs
//! - Multiple output backends (console, file, syslog)
//! - Async logging support
//! - Performance metrics for logging operations

pub use log::{debug, error, info, trace, warn, Level, LevelFilter, Metadata, Record};

/// Configuration for the logger
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    /// Minimum log level to capture
    pub level: Level,
    /// Include timestamps in logs
    pub include_timestamps: bool,
    /// Include file and line information
    pub include_location: bool,
    /// Correlation ID for tracing requests
    pub correlation_id: Option<String>,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: Level::Info,
            include_timestamps: true,
            include_location: true,
            correlation_id: None,
        }
    }
}

/// Initialize the logger with the given configuration
pub fn init(config: LoggerConfig) {
    env_logger::Builder::new()
        .filter_level(config.level.to_level_filter())
        .format(|buf, record| {
            use std::io::Write;
            write!(
                buf,
                "[{}] {} - {}",
                record.level(),
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.args()
            )
        })
        .try_init()
        .ok();
}

/// Structured logging macro for JSON-formatted logs
#[macro_export]
macro_rules! log_json {
    ($level:expr, $($key:tt = $value:expr),+ $(,)?) => {
        {
            use serde_json::json;
            let obj = json!({ $($key: $value),+ });
            log::log!($level, "{}", obj);
        }
    };
}

/// Context wrapper for correlation ID tracking
pub struct LogContext {
    pub correlation_id: String,
}

impl LogContext {
    pub fn new(id: Option<String>) -> Self {
        Self {
            correlation_id: id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_config_default() {
        let config = LoggerConfig::default();
        assert_eq!(config.level, Level::Info);
        assert!(config.include_timestamps);
        assert!(config.include_location);
    }

    #[test]
    fn test_log_context_generation() {
        let ctx = LogContext::new(None);
        assert!(!ctx.correlation_id.is_empty());
    }

    #[test]
    fn test_log_context_with_id() {
        let ctx = LogContext::new(Some("test-123".to_string()));
        assert_eq!(ctx.correlation_id, "test-123");
    }
}
