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

    // Traces to: FR-OBS-002
    #[test]
    fn test_span_id_generation() {
        let id1 = span_id();
        let id2 = span_id();
        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
        assert_ne!(id1, id2, "Span IDs should be unique");
    }

    // Traces to: FR-OBS-002
    #[test]
    fn test_trace_id_generation() {
        let id1 = trace_id();
        let id2 = trace_id();
        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
        assert_ne!(id1, id2, "Trace IDs should be unique");
    }

    // Traces to: FR-OBS-003
    #[test]
    fn test_trace_context_creation() {
        let context = TraceContext::new();
        assert!(!context.trace_id.is_empty());
        assert!(!context.span_id.is_empty());
        assert_ne!(context.trace_id, context.span_id);
    }

    // Traces to: FR-OBS-003
    #[test]
    fn test_trace_context_clone() {
        let context1 = TraceContext::new();
        let context2 = context1.clone();
        assert_eq!(context1.trace_id, context2.trace_id);
        assert_eq!(context1.span_id, context2.span_id);
    }

    // Traces to: FR-OBS-003
    #[test]
    fn test_trace_context_default() {
        let context = TraceContext::default();
        assert!(!context.trace_id.is_empty());
        assert!(!context.span_id.is_empty());
    }

    // Traces to: FR-OBS-007
    #[test]
    fn test_level_as_str_all_levels() {
        assert_eq!(level_as_str(Level::TRACE), "trace");
        assert_eq!(level_as_str(Level::DEBUG), "debug");
        assert_eq!(level_as_str(Level::INFO), "info");
        assert_eq!(level_as_str(Level::WARN), "warn");
        assert_eq!(level_as_str(Level::ERROR), "error");
    }

    // Traces to: FR-OBS-008
    #[test]
    fn test_trace_key_display() {
        let key = TraceKey("test.span");
        let display_str = format!("{}", key);
        assert_eq!(display_str, "test.span");
    }

    // Traces to: FR-OBS-008
    #[test]
    fn test_trace_key_debug() {
        let key = TraceKey("test.span");
        let debug_str = format!("{:?}", key);
        assert_eq!(debug_str, r#"TraceKey("test.span")"#);
    }

    // Traces to: FR-OBS-008
    #[test]
    fn test_trace_key_equality() {
        let key1 = TraceKey("test");
        let key2 = TraceKey("test");
        let key3 = TraceKey("other");
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    // Traces to: FR-OBS-008
    #[test]
    fn test_trace_key_hash() {
        use std::collections::HashSet;
        let key1 = TraceKey("test");
        let key2 = TraceKey("test");
        let key3 = TraceKey("other");
        let mut set = HashSet::new();
        set.insert(key1);
        assert!(set.contains(&key2));
        assert!(!set.contains(&key3));
    }

    // Traces to: FR-OBS-001
    #[test]
    fn test_config_level_validation() {
        let configs = vec![
            ("trace", "trace"),
            ("debug", "debug"),
            ("info", "info"),
            ("warn", "warn"),
            ("error", "error"),
        ];
        for (input, expected) in configs {
            let config = TracingConfig::new(input);
            assert_eq!(config.level, expected);
        }
    }

    // Traces to: FR-OBS-001
    #[test]
    fn test_config_level_with_uppercase() {
        let config = TracingConfig::new("DEBUG");
        assert_eq!(config.level, "DEBUG");
    }

    // Traces to: FR-OBS-004
    #[test]
    fn test_config_span_events() {
        let config_without = TracingConfig::new("info").with_span_events(false);
        assert!(!config_without.span_events);

        let config_with = TracingConfig::new("info").with_span_events(true);
        assert!(config_with.span_events);
    }

    // Traces to: FR-OBS-005
    #[test]
    fn test_config_thread_info() {
        let config = TracingConfig::new("info")
            .with_thread_ids(true)
            .with_thread_names(true);
        assert!(config.include_thread_ids);
        assert!(config.include_thread_names);
    }

    // Traces to: FR-OBS-005
    #[test]
    fn test_config_target_option() {
        let with_target = TracingConfig::new("info").with_target(true);
        assert!(with_target.target);

        let without_target = TracingConfig::new("info").with_target(false);
        assert!(!without_target.target);
    }

    // Traces to: FR-OBS-001
    #[test]
    fn test_builds_default_config() {
        let config = TracingConfig::default();
        assert_eq!(config.level, "info");
        assert!(!config.span_events);
        assert!(!config.include_thread_ids);
        assert!(!config.include_thread_names);
        assert!(config.target);
    }

    // Traces to: FR-OBS-001
    #[test]
    fn test_config_builders_work() {
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

    // Traces to: FR-OBS-001
    #[test]
    fn test_config_builder_partial_options() {
        let config = TracingConfig::new("warn").with_span_events(true);
        assert_eq!(config.level, "warn");
        assert!(config.span_events);
        assert!(!config.include_thread_ids);
        assert!(config.target);
    }

    // Traces to: FR-OBS-001
    #[test]
    fn test_config_clone() {
        let config1 = TracingConfig::new("debug").with_span_events(true);
        let config2 = config1.clone();
        assert_eq!(config1, config2);
    }

    // Traces to: FR-OBS-001
    #[test]
    fn test_config_equality() {
        let config1 = TracingConfig::new("info");
        let config2 = TracingConfig::new("info");
        assert_eq!(config1, config2);
    }

    // Traces to: FR-OBS-001
    #[test]
    fn test_config_inequality() {
        let config1 = TracingConfig::new("info");
        let config2 = TracingConfig::new("debug");
        assert_ne!(config1, config2);
    }
}

