//! # Phenotype Logging
//!
//! Structured logging and telemetry utilities built on top of the tracing ecosystem.
//!
//! ## Features
//!
//! - Structured JSON logging
//! - Request context tracking
//! - Performance timing spans
//! - Distributed tracing
//!
//! ## Example
//!
//! ```rust
//! use phenotype_logging::{init_logger, RequestContext};
//!
//! let _guard = init_logger();
//!
//! RequestContext::new("req-123")
//!     .with_user("alice")
//!     .scoped(|| {
//!         log::info!("Processing request");
//!     });
//! ```

use std::sync::OnceLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// Canonical tracing-init for the `phenotype-logging` crate.
///
/// Reads configuration from environment variables:
/// - `RUST_LOG` or `PHENOTYPE_LOG_LEVEL` (default: `info`)
/// - `PHENOTYPE_LOG_FORMAT` (`"json"` or `"pretty"`, default: `json`)
///
/// Safe to call multiple times — subsequent calls are silently
/// ignored (via `try_init`).
///
/// This is the single consolidated entry point; the older
/// `init_logger`, `init_logger_with_format`, and `configure_logging`
/// functions delegate to it.
pub fn init_tracing() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing_with(None, None)
}

/// Init helper that allows overriding the level and format explicitly.
/// When `level` is `Some`, it shadows the `RUST_LOG`/`PHENOTYPE_LOG_LEVEL`
/// env var.  When `format` is `Some`, it shadows the
/// `PHENOTYPE_LOG_FORMAT` env var.
pub fn init_tracing_with(
    level: Option<&str>,
    format: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let level_str = level
        .map(|s| s.to_string())
        .or_else(|| std::env::var("RUST_LOG").ok())
        .or_else(|| std::env::var("PHENOTYPE_LOG_LEVEL").ok())
        .unwrap_or_else(|| "info".to_string());

    let format_str = format
        .map(|s| s.to_string())
        .or_else(|| std::env::var("PHENOTYPE_LOG_FORMAT").ok())
        .unwrap_or_else(|| "json".to_string());

    let env_filter = EnvFilter::try_new(&level_str).unwrap_or_else(|_| EnvFilter::new("info"));

    let stdout_layer = match format_str.as_str() {
        "pretty" => tracing_subscriber::fmt::layer()
            .pretty()
            .with_filter(env_filter)
            .boxed(),
        _ => tracing_subscriber::fmt::layer()
            .json()
            .with_filter(env_filter)
            .boxed(),
    };

    let subscriber = tracing_subscriber::registry().with(stdout_layer);
    // try_init returns Err if a global subscriber is already set,
    // which we treat as a soft no-op (the first call wins).
    let _ = subscriber.try_init();
    Ok(())
}

/// Initialize the global logger (no-op if already initialized).
///
/// # Returns
/// Ok(()) if initialization succeeds, or an error if logging was already initialized
///
/// # Example
/// ```
/// use phenotype_logging::init_logger;
///
/// fn main() {
///     init_logger().expect("Failed to initialize logger");
/// }
/// ```
pub fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing()
}

/// Initialize logger with custom format.
///
/// `format` is `"pretty"` or `"json"`. Delegates to [`init_tracing_with`].
pub fn init_logger_with_format(format: &str) -> Result<(), Box<dyn std::error::Error>> {
    init_tracing_with(None, Some(format))
}

/// Request context for structured logging
#[derive(Debug, Clone)]
pub struct RequestContext {
    request_id: String,
    user_id: Option<String>,
    tenant_id: Option<String>,
    client_ip: Option<String>,
}

impl RequestContext {
    /// Create a new request context
    pub fn new(request_id: impl Into<String>) -> Self {
        Self {
            request_id: request_id.into(),
            user_id: None,
            tenant_id: None,
            client_ip: None,
        }
    }

    /// Add user ID to context
    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Add tenant ID to context
    pub fn with_tenant(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    /// Add client IP to context
    pub fn with_client_ip(mut self, ip: impl Into<String>) -> Self {
        self.client_ip = Some(ip.into());
        self
    }

    /// Get the request ID
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// Execute a function within this context scope
    pub fn scoped<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _span = tracing::info_span!(
            "request",
            request_id = %self.request_id,
            user_id = %self.user_id.as_deref().unwrap_or("anonymous"),
            tenant_id = %self.tenant_id.as_deref().unwrap_or("default"),
            client_ip = %self.client_ip.as_deref().unwrap_or("unknown"),
        );

        tracing::span::Span::current().in_scope(f)
    }
}

impl Default for RequestContext {
    fn default() -> Self {
        Self::new(uuid::Uuid::new_v4().to_string())
    }
}

/// Timing span for performance measurement
pub struct TimingSpan {
    name: String,
    start: std::time::Instant,
}

impl TimingSpan {
    /// Start a new timing span
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let span = tracing::info_span!("timing", name = %name);
        let _enter = span.enter();

        Self {
            name,
            start: std::time::Instant::now(),
        }
    }

    /// End the span and log elapsed time
    pub fn end(self) -> std::time::Duration {
        let elapsed = self.start.elapsed();
        tracing::info!(
            name = %self.name,
            elapsed_ms = elapsed.as_millis() as u64,
            "Operation completed"
        );
        elapsed
    }
}

/// Configure logging with custom filter.
///
/// `filter` is forwarded to [`init_tracing_with`] as the explicit
/// level.  This is a thin wrapper preserved for API compatibility.
pub fn configure_logging(filter: &str) {
    let _ = init_tracing_with(Some(filter), None);
}

/// Log at info level with context
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*)
    };
}

/// Log at error level with context
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*)
    };
}

/// Log at warn level with context
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*)
    };
}

/// Log at debug level with context
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*)
    };
}

// Current request context (thread-local for sync, task-local for async)
thread_local! {
    static CURRENT_CONTEXT: OnceLock<RequestContext> = OnceLock::new();
}

/// Set the current request context
pub fn set_current_context(ctx: RequestContext) {
    CURRENT_CONTEXT.with(|cell| {
        let _ = cell.set(ctx);
    });
}

/// Get the current request context if set
pub fn current_context() -> Option<RequestContext> {
    CURRENT_CONTEXT.with(|cell| cell.get().cloned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_context() {
        let ctx = RequestContext::new("test-123")
            .with_user("alice")
            .with_tenant("acme");

        assert_eq!(ctx.request_id(), "test-123");
    }

    #[test]
    fn test_timing_span() {
        let span = TimingSpan::new("test_operation");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = span.end();
        assert!(elapsed.as_millis() >= 10);
    }

    #[test]
    fn test_default_context() {
        let ctx = RequestContext::default();
        // Should have a UUID as request_id
        assert!(!ctx.request_id().is_empty());
        assert_ne!(ctx.request_id(), "test-123"); // Different from explicit
    }
}
