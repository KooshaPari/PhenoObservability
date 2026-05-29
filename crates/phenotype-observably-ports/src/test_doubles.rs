//! In-memory test doubles for [`CachePort`] and [`TimeSeriesPort`].
//!
//! Available only with the `test-util` Cargo feature.

use crate::cache::{CachePort, CacheResult};
use crate::timeseries::{TimeSeriesPort, TsLogEntry, TsMetric, TsResult};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// InMemoryCache
// ---------------------------------------------------------------------------

/// Thread-safe in-memory implementation of [`CachePort`].
///
/// Entries never actually expire — this double is for logic tests only.
/// Use a real Dragonfly/Redis instance for TTL-dependent tests.
#[derive(Clone, Default)]
pub struct InMemoryCache {
    store: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl InMemoryCache {
    /// Inspect the raw store (useful in assertions).
    pub fn snapshot(&self) -> HashMap<String, Vec<u8>> {
        self.store.lock().unwrap().clone()
    }
}

#[async_trait]
impl CachePort for InMemoryCache {
    async fn get(&self, key: &str) -> CacheResult<Option<Vec<u8>>> {
        Ok(self.store.lock().unwrap().get(key).cloned())
    }

    async fn set(&self, key: &str, value: &[u8], _ttl_seconds: u64) -> CacheResult<()> {
        self.store
            .lock()
            .unwrap()
            .insert(key.to_owned(), value.to_vec());
        Ok(())
    }

    async fn delete(&self, key: &str) -> CacheResult<()> {
        self.store.lock().unwrap().remove(key);
        Ok(())
    }

    async fn expire(&self, key: &str, _ttl_seconds: u64) -> CacheResult<bool> {
        Ok(self.store.lock().unwrap().contains_key(key))
    }
}

// ---------------------------------------------------------------------------
// InMemoryTimeSeries
// ---------------------------------------------------------------------------

/// In-memory implementation of [`TimeSeriesPort`].
///
/// All ingested rows accumulate in `metrics` / `logs` until flushed; after
/// flush they move to `flushed_metrics` / `flushed_logs`.  Tests can
/// inspect either collection.
#[derive(Default)]
pub struct InMemoryTimeSeries {
    pending_metrics: Vec<TsMetric>,
    pending_logs: Vec<TsLogEntry>,
    pub flushed_metrics: Vec<TsMetric>,
    pub flushed_logs: Vec<TsLogEntry>,
}

impl InMemoryTimeSeries {
    /// Total number of rows that have been flushed.
    pub fn flushed_count(&self) -> usize {
        self.flushed_metrics.len() + self.flushed_logs.len()
    }
}

#[async_trait]
impl TimeSeriesPort for InMemoryTimeSeries {
    async fn ingest_metric(&mut self, metric: TsMetric) -> TsResult<()> {
        self.pending_metrics.push(metric);
        Ok(())
    }

    async fn ingest_log(&mut self, log: TsLogEntry) -> TsResult<()> {
        self.pending_logs.push(log);
        Ok(())
    }

    async fn flush(&mut self) -> TsResult<usize> {
        let count = self.pending_metrics.len() + self.pending_logs.len();
        self.flushed_metrics.extend(self.pending_metrics.drain(..));
        self.flushed_logs.extend(self.pending_logs.drain(..));
        Ok(count)
    }

    fn pending(&self) -> usize {
        self.pending_metrics.len() + self.pending_logs.len()
    }
}

// ---------------------------------------------------------------------------
// Unit tests (always compiled — no network required)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeseries::{TsLogEntry, TsMetric};
    use chrono::Utc;
    use std::collections::HashMap;

    // --- CachePort object-safety sanity ---
    #[test]
    fn cache_port_is_object_safe() {
        let c: Box<dyn CachePort> = Box::new(InMemoryCache::default());
        drop(c);
    }

    // --- TimeSeriesPort object-safety sanity ---
    #[test]
    fn ts_port_is_object_safe() {
        let ts: Box<dyn TimeSeriesPort> = Box::new(InMemoryTimeSeries::default());
        drop(ts);
    }

    // --- InMemoryCache round-trip ---
    #[tokio::test]
    async fn cache_set_get_delete() {
        let cache = InMemoryCache::default();

        cache.set("hello", b"world", 30).await.unwrap();
        assert_eq!(cache.get("hello").await.unwrap(), Some(b"world".to_vec()));

        cache.delete("hello").await.unwrap();
        assert_eq!(cache.get("hello").await.unwrap(), None);
    }

    #[tokio::test]
    async fn cache_expire_returns_false_for_missing_key() {
        let cache = InMemoryCache::default();
        let existed = cache.expire("no-such-key", 60).await.unwrap();
        assert!(!existed);
    }

    #[tokio::test]
    async fn cache_expire_returns_true_for_present_key() {
        let cache = InMemoryCache::default();
        cache.set("k", b"v", 10).await.unwrap();
        let existed = cache.expire("k", 60).await.unwrap();
        assert!(existed);
    }

    // --- InMemoryTimeSeries round-trip ---
    fn make_metric(name: &str, value: f64) -> TsMetric {
        TsMetric {
            timestamp: Utc::now(),
            name: name.to_owned(),
            value,
            labels: HashMap::new(),
        }
    }

    fn make_log(msg: &str) -> TsLogEntry {
        TsLogEntry {
            timestamp: Utc::now(),
            level: "INFO".to_owned(),
            message: msg.to_owned(),
            source: "test".to_owned(),
            trace_id: None,
            labels: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn ts_ingest_and_flush() {
        let mut ts = InMemoryTimeSeries::default();

        ts.ingest_metric(make_metric("cpu", 0.5)).await.unwrap();
        ts.ingest_log(make_log("hello")).await.unwrap();
        assert_eq!(ts.pending(), 2);

        let flushed = ts.flush().await.unwrap();
        assert_eq!(flushed, 2);
        assert_eq!(ts.pending(), 0);
        assert_eq!(ts.flushed_count(), 2);
    }

    #[tokio::test]
    async fn ts_flush_empty_is_zero() {
        let mut ts = InMemoryTimeSeries::default();
        let flushed = ts.flush().await.unwrap();
        assert_eq!(flushed, 0);
    }

    #[tokio::test]
    async fn ts_multiple_flush_accumulates() {
        let mut ts = InMemoryTimeSeries::default();
        ts.ingest_metric(make_metric("m1", 1.0)).await.unwrap();
        ts.flush().await.unwrap();
        ts.ingest_metric(make_metric("m2", 2.0)).await.unwrap();
        ts.flush().await.unwrap();
        assert_eq!(ts.flushed_metrics.len(), 2);
    }
}
