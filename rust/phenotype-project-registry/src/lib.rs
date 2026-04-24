//! Placeholder stub crate for `phenotype-project-registry`.
//!
//! Provides the minimal type surface consumed by `phenotype-health-cli`:
//! `ProjectType`, `ProjectMetadata`, and `discover_projects`. The stub
//! `discover_projects` returns an empty list; real discovery logic lives in
//! the future full implementation.
//!
//! Tracks: PhenoObservability precursor 3 (missing-crate deps) → issue #50.

use serde::Serialize;
use std::path::{Path, PathBuf};

/// Project type classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ProjectType {
    RustLibrary,
    RustBinary,
    TypeScriptLibrary,
    TypeScriptApplication,
    GoModule,
    PythonPackage,
    Unknown,
}

/// Health-config surface embedded in project metadata.
#[derive(Debug, Clone, Default, Serialize)]
pub struct HealthConfig {
    pub enabled: bool,
}

/// Metadata describing a single discovered project.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectMetadata {
    pub name: String,
    pub path: PathBuf,
    pub project_type: ProjectType,
    pub health_config: HealthConfig,
}

/// Errors returned by registry discovery.
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("discovery failed: {0}")]
    Other(String),
}

/// Discover projects under a given root path.
///
/// Stub: always returns `Ok(Vec::new())`. The real implementation will walk
/// the filesystem looking for `Cargo.toml`, `package.json`, `go.mod`, and
/// `pyproject.toml` markers.
pub fn discover_projects(_root: impl AsRef<Path>) -> Result<Vec<ProjectMetadata>, RegistryError> {
    Ok(Vec::new())
}
