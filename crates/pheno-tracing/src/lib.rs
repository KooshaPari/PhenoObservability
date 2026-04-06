use std::fmt;

use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{prelude::*, EnvFilter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TracingConfig {
    pub level: String,
    pub span_events: bool,
    pub include_thread_ids: bool,
    pub include_thread_names: bool,
    pub target: bool,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            span_events: false,
            include_thread_ids: false,
            include_thread_names: false,
            target: true,
        }
    }
}

impl TracingConfig {
    pub fn new(level: impl Into<String>) -> Self {
        Self {
            level: level.into(),
            ..Self::default()
        }
    }

    pub fn with_span_events(mut self, span_events: bool) -> Self {
        self.span_events = span_events;
        self
    }

    pub fn with_thread_ids(mut self, include_thread_ids: bool) -> Self {
        self.include_thread_ids = include_thread_ids;
        self
    }

    pub fn with_thread_names(mut self, include_thread_names: bool) -> Self {
        self.include_thread_names = include_thread_names;
        self
    }

    pub fn with_target(mut self, target: bool) -> Self {
        self.target = target;
        self
    }
}

pub fn init_tracing(
    config: TracingConfig,
) -> Result<(), tracing_subscriber::util::TryInitError> {
    build_subscriber(&config).try_init()
}

pub fn build_subscriber(
    config: &TracingConfig,
) -> impl tracing::Subscriber + Send + Sync + 'static {
    let filter = EnvFilter::try_new(config.level.as_str()).unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(config.target)
        .with_thread_ids(config.include_thread_ids)
        .with_thread_names(config.include_thread_names)
        .with_span_events(if config.span_events {
            FmtSpan::FULL
        } else {
            FmtSpan::NONE
        });

    tracing_subscriber::registry().with(filter).with(fmt_layer)
}

pub fn span_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn trace_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn level_as_str(level: Level) -> &'static str {
    match level {
        Level::TRACE => "trace",
        Level::DEBUG => "debug",
        Level::INFO => "info",
        Level::WARN => "warn",
        Level::ERROR => "error",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TraceKey<'a>(pub &'a str);

impl fmt::Display for TraceKey<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
}

impl TraceContext {
    pub fn new() -> Self {
        Self {
            trace_id: trace_id(),
            span_id: span_id(),
        }
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_span_ids() {
        let id = span_id();
        assert!(!id.is_empty());
    }

    #[test]
    fn builds_trace_context() {
        let context = TraceContext::new();
        assert!(!context.trace_id.is_empty());
        assert!(!context.span_id.is_empty());
        assert_ne!(context.trace_id, context.span_id);
    }

    #[test]
    fn maps_levels() {
        assert_eq!(level_as_str(Level::INFO), "info");
        assert_eq!(level_as_str(Level::ERROR), "error");
    }

    #[test]
    fn builds_default_config() {
        let config = TracingConfig::default();
        assert_eq!(config.level, "info");
        assert!(!config.span_events);
        assert!(config.target);
    }

    #[test]
    fn config_builders_work() {
        let config = TracingConfig::new("debug")
            .with_span_events(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_target(false);

        assert_eq!(config.level, "debug");
        assert!(config.span_events);
        assert!(config.include_thread_ids);
        assert!(config.include_thread_names);
        assert!(!config.target);
    }
}

