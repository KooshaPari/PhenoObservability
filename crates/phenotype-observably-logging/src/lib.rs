//! Structured logging with context propagation (helix-logging patterns).
//!
//! `LogContext` is re-exported from `tracely` — the canonical location
//! for the type since the 2026-03-26 absorption of `helix-logging`.

/// Re-exported from `tracely` to keep a single source of truth for
/// `LogContext` across the workspace.
pub use tracely::LogContext;

pub struct StructuredLogger {
    #[allow(dead_code)]
    context: LogContext,
}

impl StructuredLogger {
    pub fn new(context: LogContext) -> Self {
        Self { context }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_context_creation() {
        // Exercise the canonical LogContext API from `tracely`.
        let ctx = LogContext::new(Some("trace-1".to_string()));
        assert_eq!(ctx.correlation_id, "trace-1");
    }
}
