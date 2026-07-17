//! Structured logging with context propagation (helix-logging patterns).

/// Structured log context propagated across service boundaries.
pub struct LogContext {
    pub trace_id: String,
    pub span_id: String,
    pub service: String,
}

/// Structured logger bound to a [`LogContext`].
pub struct StructuredLogger {
    pub(crate) context: LogContext,
}

/// Construct a structured logger for the given context.
pub fn new_structured_logger(context: LogContext) -> StructuredLogger {
    StructuredLogger { context }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_context_creation() {
        let ctx = LogContext {
            trace_id: "trace-1".to_string(),
            span_id: "span-1".to_string(),
            service: "api".to_string(),
        };
        assert_eq!(ctx.service, "api");
    }

    #[test]
    fn test_structured_logger_retains_context() {
        let ctx = LogContext {
            trace_id: "trace-2".to_string(),
            span_id: "span-2".to_string(),
            service: "worker".to_string(),
        };
        let logger = new_structured_logger(ctx);
        assert_eq!(logger.context.service, "worker");
    }
}
