//! Hexagonal port traits for PhenoObservability adapters.
//!
//! This crate defines the **driver-side ports** (interfaces) that infra
//! adapters must implement so consumers can program against abstractions and
//! swap real adapters for in-memory test doubles.
//!
//! # Ports
//! - [`CachePort`]       — generic key/value cache (Dragonfly / Redis adapter)
//! - [`TimeSeriesPort`]  — time-series ingest (QuestDB adapter)
//!
//! # Test doubles (feature `test-util`)
//! Enable the `test-util` Cargo feature to get:
//! - [`test_doubles::InMemoryCache`]
//! - [`test_doubles::InMemoryTimeSeries`]

pub mod cache;
pub mod timeseries;

#[cfg(feature = "test-util")]
pub mod test_doubles;

pub use cache::CachePort;
pub use timeseries::TimeSeriesPort;
