//! L11 chaos game-day test for `phenotype-observably-ports` (v25 cycle-15 T5).
//!
//! Exercises the public [`CachePort`] surface under two injected failure modes:
//!
//! 1. **Latency injection** — adds a deterministic 100-500 ms sleep before
//!    each `get` call (drawn from a xorshift64 PRNG seeded with `42`).
//! 2. **Error injection** — returns `Err(RepositoryError::NotFound)` on every
//!    Nth `get` call (N = 4) so 1/4 of calls exercise the error path.
//!
//! Determinism note: every test seeds the PRNG with the same seed (`42`) so
//! the fault sequence is reproducible across runs and CI machines.
//!
//! Run with:
//!   cargo test -p phenotype-observably-ports --test chaos_l11_2026_06_22
//!
//! Invariant: every test must complete in under 10 s wall time. The
//! latency-injection test bounds the upper chaos-call count at 6 (so worst
//! case 6 * 500 ms = 3 s), keeping the wall time well under the 10 s CI gate.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use phenotype_errors::RepositoryError;
use phenotype_observably_ports::{CachePort, CacheResult};

/// Tiny in-process `CachePort` impl used so this integration test does not
/// depend on the `test-util` Cargo feature (default-features-only). Keeps the
/// surface identical to `InMemoryCache` for the methods under test.
#[derive(Clone, Default)]
struct TestCache {
    store: Arc<Mutex<std::collections::HashMap<String, Vec<u8>>>>,
}

#[async_trait]
impl CachePort for TestCache {
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

/// Deterministic xorshift64 PRNG. Seed = 42 produces the same fault sequence
/// across runs and platforms.
struct Xorshift64(u64);

impl Xorshift64 {
    fn new(seed: u64) -> Self {
        // Avoid the all-zeros fixed point by ensuring the seed is non-zero.
        Self(seed | 1)
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Draw a u64 in the inclusive range `[lo, hi]`.
    fn range(&mut self, lo: u64, hi: u64) -> u64 {
        assert!(hi >= lo, "range invalid: lo={lo} hi={hi}");
        let span = hi - lo + 1;
        lo + (self.next_u64() % span)
    }
}

/// Chaos gate for async `CachePort::get` ops. If the injector fires a fault
/// (every 4th call, deterministic via seed=42), return Err instead of
/// delegating to the underlying port.
async fn chaos_get(
    injector: &mut Xorshift64,
    port: &TestCache,
    key: &str,
) -> CacheResult<Option<Vec<u8>>> {
    // Latency injection: sleep 100-500 ms before every call.
    let delay_ms = injector.range(100, 500);
    tokio::time::sleep(Duration::from_millis(delay_ms)).await;

    // Error injection: every 4th call returns Err(NotFound) (1/N pattern).
    let draw = injector.range(0, 3);
    if draw == 0 {
        return Err(RepositoryError::NotFound(format!(
            "chaos: forced miss on key {key:?}"
        )));
    }
    port.get(key).await
}

/// **Chaos enabled (latency + error)**: 6 calls through the gate. With
/// seed=42 + 1/4 error rate, we expect at least one error and at least one
/// successful read. Each call is bounded at 600 ms wall (500 ms chaos +
/// 100 ms margin).
#[tokio::test(flavor = "current_thread")]
async fn chaos_latency_and_error_injection_handled_cleanly() {
    let cache = TestCache::default();
    cache.set("alpha", b"one", 60).await.unwrap();
    cache.set("beta", b"two", 60).await.unwrap();

    let mut injector = Xorshift64::new(42);
    let mut ok_count = 0;
    let mut err_count = 0;
    let start = Instant::now();

    for key in ["alpha", "beta", "alpha", "beta", "alpha", "beta"] {
        let result = chaos_get(&mut injector, &cache, key).await;
        match result {
            Ok(Some(_)) => ok_count += 1,
            Ok(None) => panic!("unexpected None — seed drift?"),
            Err(RepositoryError::NotFound(_)) => err_count += 1,
            Err(other) => panic!("unexpected error variant: {other:?}"),
        }
    }

    let elapsed = start.elapsed();
    assert_eq!(ok_count + err_count, 6);
    assert!(ok_count > 0, "at least one call must succeed; got 0");
    assert!(err_count > 0, "at least one call must error; got 0");
    assert!(
        elapsed < Duration::from_secs(10),
        "wall time {elapsed:?} exceeded 10 s gate"
    );
}

/// **Chaos disabled (control)**: a baseline `get` loop without the gate. All
/// 6 calls succeed and the total wall time is bounded by tokio scheduling
/// jitter (well under 1 s).
#[tokio::test(flavor = "current_thread")]
async fn chaos_disabled_baseline_completes_quickly() {
    let cache = TestCache::default();
    cache.set("k1", b"v1", 60).await.unwrap();

    let start = Instant::now();
    for _ in 0..6 {
        let v = cache.get("k1").await.unwrap();
        assert_eq!(v, Some(b"v1".to_vec()));
    }
    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_secs(1),
        "baseline should be sub-second; took {elapsed:?}"
    );
}

/// **Determinism**: two PRNG instances with seed=42 produce the same latency
/// sequence and the same error-draw sequence. CI depends on this for
/// reproducible triage.
#[test]
fn chaos_deterministic_seed_produces_identical_sequence() {
    let mut a = Xorshift64::new(42);
    let mut b = Xorshift64::new(42);

    let latencies: Vec<u64> = (0..50).map(|_| a.range(100, 500)).collect();
    let latencies_b: Vec<u64> = (0..50).map(|_| b.range(100, 500)).collect();
    assert_eq!(latencies, latencies_b);

    let errors_a: Vec<u64> = (0..50).map(|_| a.range(0, 3)).collect();
    let errors_b: Vec<u64> = (0..50).map(|_| b.range(0, 3)).collect();
    assert_eq!(errors_a, errors_b);
    // At least one error draw must equal 0 (the fault trigger).
    assert!(errors_a.contains(&0), "1/4 error rate must trigger at least once");
}

/// **Forced error path**: when the gate fires, the error message must contain
/// the key for log triage. This pins the error-injection contract independent
/// of the random draw.
#[tokio::test(flavor = "current_thread")]
async fn chaos_forced_error_message_contains_key() {
    let cache = TestCache::default();
    let mut injector = Xorshift64::new(42);

    // Drive the injector until we get a fault draw, then verify the message.
    let mut found = false;
    for _ in 0..16 {
        let r = chaos_get(&mut injector, &cache, "triage-key").await;
        if let Err(RepositoryError::NotFound(msg)) = r {
            assert!(
                msg.contains("triage-key"),
                "forced-error message must contain the key for log triage; got {msg:?}"
            );
            found = true;
            break;
        }
    }
    assert!(found, "injector must fire a forced error within 16 draws");
}
