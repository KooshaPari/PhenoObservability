//! Placeholder stub crate for `phenotype-security-aggregator`.
//!
//! Minimal no-op API shaped to match the surface consumed by
//! `phenotype-health-cli`. The full aggregator (CVE feeds, vulnerability
//! databases, exploit intel) is out of scope for the missing-deps precursor.
//!
//! Tracks: PhenoObservability precursor 3 (missing-crate deps) → issue #50.

use serde::Serialize;

/// Finding severity used by the aggregator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// A security finding aggregated from one or more sources.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub id: String,
    pub title: String,
    pub severity: Severity,
    pub source: String,
    pub description: Option<String>,
}

/// Aggregated report returned by `SecurityAggregator::aggregate`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct AggregatedReport {
    pub findings: Vec<Finding>,
}

/// Security aggregator. Stub always returns an empty report.
#[derive(Debug, Clone, Default)]
pub struct SecurityAggregator {
    _private: (),
}

impl SecurityAggregator {
    /// Construct a new aggregator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Aggregate findings from all configured sources.
    ///
    /// Stub: always returns an empty `AggregatedReport`.
    pub async fn aggregate(&self) -> Result<AggregatedReport, AggregateError> {
        Ok(AggregatedReport::default())
    }
}

/// Errors returned by the aggregator.
#[derive(Debug, thiserror::Error)]
pub enum AggregateError {
    #[error("aggregation failed: {0}")]
    Other(String),
}
