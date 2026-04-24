//! Placeholder stub crate for `phenotype-compliance-scanner`.
//!
//! This crate exists so that `phenotype-health-cli` can reference a concrete
//! dependency while the full compliance scanner is designed and implemented
//! elsewhere. It provides a minimal no-op API surface matching the shape
//! consumed by `phenotype-health-cli` so the workspace dedupe precursor can
//! land cleanly.
//!
//! Tracks: PhenoObservability precursor 3 (missing-crate deps) → issue #50.

use serde::Serialize;
use std::path::Path;

/// Finding severity. Matches the shape consumed by `phenotype-health-cli`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// A compliance finding reported by the scanner.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub rule_id: String,
    pub message: String,
    pub severity: Severity,
    pub path: Option<String>,
}

/// Result of a scan over a single project path.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ScanResult {
    pub findings: Vec<Finding>,
}

/// Compliance scanner. This stub returns an empty result for every scan.
#[derive(Debug, Clone, Default)]
pub struct ComplianceScanner {
    _private: (),
}

impl ComplianceScanner {
    /// Construct a new scanner.
    pub fn new() -> Self {
        Self::default()
    }

    /// Scan a project path. Stub: always returns an empty `ScanResult`.
    pub fn scan(&self, _path: impl AsRef<Path>) -> Result<ScanResult, ScanError> {
        Ok(ScanResult::default())
    }
}

/// Errors that can be returned by the scanner.
#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("scan failed: {0}")]
    Other(String),
}
