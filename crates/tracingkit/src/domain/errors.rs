//! Domain Errors

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TraceError {
    #[error("Export error: {0}")]
    Export(String),

    #[error("Context error: {0}")]
    Context(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

pub type TraceResult<T> = Result<T, TraceError>;
