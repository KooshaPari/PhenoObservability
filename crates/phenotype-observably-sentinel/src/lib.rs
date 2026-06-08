//! Sentinel patterns: circuit breaker, bulkhead, and threshold-based
//! alerting rule engine.
//!
//! Rate limiting is **re-exported** from `phenotype-sentinel`
//! (the `tracely-sentinel` crate) — the canonical, more featureful
//! implementation (TokenBucket + LeakyBucket + trait + config
//! validation).  See [`RateLimiter`], [`TokenBucket`], and
//! [`LeakyBucket`] for the rate-limiter facade.
//!
//! See [`alerting`] for the hexagonal alerting port and rule evaluator.
//!
//! # Circuit Breaker
//! Three-state FSM (Closed → Open → Half-Open) backed by [`parking_lot`]
//! `RwLock`.  Wraps any fallible closure; counts failures and trips the
//! breaker when `failure_threshold` is exceeded within the current window.
//!
//! # Bulkhead
//! Counting semaphore that caps concurrent in-flight calls.

pub mod alerting;

pub use alerting::{
    Alert, AlertError, AlertEvaluator, AlertRule, AlertSink, InMemoryAlertSink, LogAlertSink,
    MetricSample, Severity, ThresholdOp,
};

// ── Rate-limiter facade (re-exported from `phenotype-sentinel`) ────────────
//
// The previous governor-based `Sentinel` rate limiter has been removed in
// favour of the more featureful `phenotype-sentinel` crate (which provides
// TokenBucket, LeakyBucket, the `RateLimiter` trait, async acquire, and
// config validation).  The re-exports below keep a single import path
// for consumers that previously used `phenotype_observably_sentinel::Sentinel`.

pub use phenotype_sentinel::rate_limiter::RateLimiterError;
pub use phenotype_sentinel::{
    config, LeakyBucket, RateLimiter, RateLimiterConfig, TokenBucket,
};

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{
    sync::{
        atomic::{AtomicU32, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use thiserror::Error;

// ── Errors ────────────────────────────────────────────────────────────────────
//
// `SentinelError` covers the remaining sentinel patterns (circuit breaker,
// bulkhead).  The rate-limit error variant was removed along with the
// governor-based `Sentinel` rate limiter — see `phenotype_sentinel::RateLimiterError`.

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SentinelError {
    #[error("circuit breaker is open")]
    CircuitOpen,
    #[error("bulkhead capacity exhausted (max {0} concurrent)")]
    BulkheadFull(u32),
}


// ── Circuit Breaker ───────────────────────────────────────────────────────────

/// Circuit-breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation; failures are counted.
    Closed,
    /// Too many failures; calls are rejected immediately.
    Open,
    /// Breaker allows one probe call to test recovery.
    HalfOpen,
}

/// Configuration for the circuit breaker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before the breaker opens.
    pub failure_threshold: u32,
    /// How long (ms) the breaker stays Open before moving to Half-Open.
    pub open_duration_ms: u64,
}

struct CbInner {
    state: CircuitState,
    failure_count: u32,
    opened_at: Option<Instant>,
}

/// Thread-safe three-state circuit breaker.
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    inner: RwLock<CbInner>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Arc<Self> {
        Arc::new(Self {
            config,
            inner: RwLock::new(CbInner {
                state: CircuitState::Closed,
                failure_count: 0,
                opened_at: None,
            }),
        })
    }

    /// Return the current circuit state (may transition Open→HalfOpen on read
    /// if the open duration has elapsed).
    pub fn state(&self) -> CircuitState {
        self.maybe_transition();
        self.inner.read().state
    }

    /// Execute `f`.  Returns `Err(SentinelError::CircuitOpen)` when the breaker
    /// is open.  On success the failure counter resets; on failure it increments
    /// and may trip the breaker.
    pub fn call<F, T, E>(&self, f: F) -> Result<T, SentinelError>
    where
        F: FnOnce() -> Result<T, E>,
    {
        self.maybe_transition();

        {
            let state = self.inner.read().state;
            if state == CircuitState::Open {
                return Err(SentinelError::CircuitOpen);
            }
        }

        match f() {
            Ok(v) => {
                self.on_success();
                Ok(v)
            }
            Err(_) => {
                self.on_failure();
                Err(SentinelError::CircuitOpen)
            }
        }
    }

    // ── private helpers ───────────────────────────────────────────────────────

    fn maybe_transition(&self) {
        // Promote Open → HalfOpen once the open_duration elapses.
        // Use a read-lock fast path first.
        let should_transition = {
            let g = self.inner.read();
            if g.state == CircuitState::Open {
                if let Some(opened_at) = g.opened_at {
                    opened_at.elapsed()
                        >= Duration::from_millis(self.config.open_duration_ms)
                } else {
                    false
                }
            } else {
                false
            }
        };

        if should_transition {
            let mut g = self.inner.write();
            // Double-check under write lock.
            if g.state == CircuitState::Open {
                if let Some(opened_at) = g.opened_at {
                    if opened_at.elapsed()
                        >= Duration::from_millis(self.config.open_duration_ms)
                    {
                        g.state = CircuitState::HalfOpen;
                    }
                }
            }
        }
    }

    fn on_success(&self) {
        let mut g = self.inner.write();
        g.failure_count = 0;
        g.state = CircuitState::Closed;
        g.opened_at = None;
    }

    fn on_failure(&self) {
        let mut g = self.inner.write();
        g.failure_count += 1;
        if g.state == CircuitState::HalfOpen
            || g.failure_count >= self.config.failure_threshold
        {
            g.state = CircuitState::Open;
            g.opened_at = Some(Instant::now());
        }
    }
}

// ── Bulkhead ──────────────────────────────────────────────────────────────────

/// Counting-semaphore bulkhead that caps concurrent in-flight calls.
#[derive(Debug)]
pub struct Bulkhead {
    capacity: u32,
    in_flight: AtomicU32,
    total_accepted: AtomicU64,
    total_rejected: AtomicU64,
}

/// RAII guard that decrements the in-flight counter on drop.
#[derive(Debug)]
pub struct BulkheadGuard<'a> {
    bulkhead: &'a Bulkhead,
}

impl Drop for BulkheadGuard<'_> {
    fn drop(&mut self) {
        self.bulkhead.in_flight.fetch_sub(1, Ordering::AcqRel);
    }
}

impl Bulkhead {
    /// Create a new bulkhead with `capacity` concurrent slots.
    pub fn new(capacity: u32) -> Self {
        Self {
            capacity,
            in_flight: AtomicU32::new(0),
            total_accepted: AtomicU64::new(0),
            total_rejected: AtomicU64::new(0),
        }
    }

    /// Try to acquire a slot.  Returns a [`BulkheadGuard`] that releases the
    /// slot on drop, or `Err(SentinelError::BulkheadFull)`.
    pub fn acquire(&self) -> Result<BulkheadGuard<'_>, SentinelError> {
        // CAS-loop to claim a slot only if capacity is not exhausted.
        let mut current = self.in_flight.load(Ordering::Acquire);
        loop {
            if current >= self.capacity {
                self.total_rejected.fetch_add(1, Ordering::Relaxed);
                return Err(SentinelError::BulkheadFull(self.capacity));
            }
            match self.in_flight.compare_exchange_weak(
                current,
                current + 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    self.total_accepted.fetch_add(1, Ordering::Relaxed);
                    return Ok(BulkheadGuard { bulkhead: self });
                }
                Err(actual) => current = actual,
            }
        }
    }

    /// Current number of in-flight calls.
    pub fn in_flight(&self) -> u32 {
        self.in_flight.load(Ordering::Acquire)
    }

    /// Cumulative accepted count.
    pub fn total_accepted(&self) -> u64 {
        self.total_accepted.load(Ordering::Relaxed)
    }

    /// Cumulative rejected count.
    pub fn total_rejected(&self) -> u64 {
        self.total_rejected.load(Ordering::Relaxed)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    // ── CircuitBreaker ───────────────────────────────────────────────────────

    fn ok_call() -> Result<u32, &'static str> {
        Ok(42)
    }
    fn err_call() -> Result<u32, &'static str> {
        Err("boom")
    }

    #[test]
    fn circuit_breaker_starts_closed() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            open_duration_ms: 500,
        });
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn circuit_breaker_success_keeps_closed() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 2,
            open_duration_ms: 100,
        });
        assert!(cb.call(ok_call).is_ok());
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn circuit_breaker_trips_after_threshold() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 2,
            open_duration_ms: 5000,
        });
        // Two failures → breaker opens.
        let _ = cb.call(err_call);
        let _ = cb.call(err_call);
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn circuit_breaker_rejects_while_open() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            open_duration_ms: 5000,
        });
        let _ = cb.call(err_call); // trips breaker
        // Next call must be rejected without invoking the closure.
        assert_eq!(
            cb.call(ok_call),
            Err(SentinelError::CircuitOpen),
            "open breaker must reject calls"
        );
    }

    #[test]
    fn circuit_breaker_resets_on_success_after_half_open() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            open_duration_ms: 1, // 1 ms → transitions to HalfOpen almost immediately
        });
        let _ = cb.call(err_call); // open
        // Wait for open_duration.
        thread::sleep(Duration::from_millis(5));
        // Should now be HalfOpen; a success resets to Closed.
        assert!(cb.call(ok_call).is_ok());
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn circuit_breaker_failure_in_half_open_reopens() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            open_duration_ms: 1,
        });
        let _ = cb.call(err_call); // open
        thread::sleep(Duration::from_millis(5));
        // HalfOpen → failure → Open again.
        let _ = cb.call(err_call);
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn circuit_breaker_failure_counter_resets_on_success() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            open_duration_ms: 5000,
        });
        let _ = cb.call(err_call);
        let _ = cb.call(err_call);
        // One success before threshold → should reset counter.
        let _ = cb.call(ok_call);
        // Need 3 more failures now (counter was reset).
        let _ = cb.call(err_call);
        let _ = cb.call(err_call);
        assert_eq!(cb.state(), CircuitState::Closed, "counter should have reset");
        let _ = cb.call(err_call);
        assert_eq!(cb.state(), CircuitState::Open);
    }

    // ── Bulkhead ─────────────────────────────────────────────────────────────

    #[test]
    fn bulkhead_accepts_up_to_capacity() {
        let bh = Bulkhead::new(3);
        let _g1 = bh.acquire().unwrap();
        let _g2 = bh.acquire().unwrap();
        let _g3 = bh.acquire().unwrap();
        assert_eq!(bh.in_flight(), 3);
    }

    #[test]
    fn bulkhead_rejects_beyond_capacity() {
        let bh = Bulkhead::new(2);
        let _g1 = bh.acquire().unwrap();
        let _g2 = bh.acquire().unwrap();
        assert_eq!(
            bh.acquire().unwrap_err(),
            SentinelError::BulkheadFull(2)
        );
    }

    #[test]
    fn bulkhead_guard_releases_on_drop() {
        let bh = Bulkhead::new(1);
        {
            let _g = bh.acquire().unwrap();
            assert_eq!(bh.in_flight(), 1);
        } // drop
        assert_eq!(bh.in_flight(), 0, "guard drop should release the slot");
        // Slot is free again.
        assert!(bh.acquire().is_ok());
    }

    #[test]
    fn bulkhead_tracks_accepted_and_rejected_counts() {
        let bh = Bulkhead::new(1);
        let g = bh.acquire().unwrap(); // accepted 1
        let _ = bh.acquire(); // rejected 1
        drop(g);
        let _ = bh.acquire(); // accepted 2

        assert_eq!(bh.total_accepted(), 2);
        assert_eq!(bh.total_rejected(), 1);
    }

    #[test]
    fn bulkhead_concurrent_contention() {
        use std::sync::Arc;
        let bh = Arc::new(Bulkhead::new(4));
        let mut handles = vec![];

        for _ in 0..8 {
            let bh = Arc::clone(&bh);
            handles.push(thread::spawn(move || {
                // Try to grab a slot; ignore the result.
                let _guard = bh.acquire();
                thread::sleep(Duration::from_millis(1));
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        // All guards dropped; in-flight must be 0.
        assert_eq!(bh.in_flight(), 0);
        // accepted + rejected == 8 spawned threads
        assert_eq!(bh.total_accepted() + bh.total_rejected(), 8);
    }

    // ── SentinelError display ────────────────────────────────────────────────

    #[test]
    fn error_display_messages() {
        // `RateLimitExceeded` was removed with the governor-based Sentinel
        // rate limiter; use `phenotype_sentinel::RateLimiterError` instead.
        assert_eq!(
            SentinelError::CircuitOpen.to_string(),
            "circuit breaker is open"
        );
        assert_eq!(
            SentinelError::BulkheadFull(5).to_string(),
            "bulkhead capacity exhausted (max 5 concurrent)"
        );
    }
}
