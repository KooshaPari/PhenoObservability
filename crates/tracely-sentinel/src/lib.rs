//! # phenotype-sentinel
//!
//! Rust resilience library providing rate limiting, circuit breaking, and bulkhead isolation.
//!
//! ## Features
//!
//! - **Rate Limiting**: Token bucket and leaky bucket algorithms
//! - **Circuit Breaker**: Failure threshold-based circuit breaker pattern
//! - **Bulkhead**: Partition-based isolation for concurrent operations
//!
//! ## Quick Start
//!
//! ```rust
//! use phenotype_sentinel::{RateLimiter, TokenBucket};
//!
//! let mut limiter = TokenBucket::new(100, 10); // 100 tokens, refill 10/sec
//! if limiter.try_acquire() {
//!     // proceed with request
//! }
//! ```

/// @trace SENT-001
pub mod bulkhead;
pub mod circuit_breaker;
pub mod config;
pub mod rate_limiter;
pub mod validation;

pub use bulkhead::{Bulkhead, PartitionGuard};
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerError, CircuitState};
pub use config::{BulkheadConfig, CircuitBreakerConfig, RateLimiterConfig, SentinelConfig};
pub use rate_limiter::{LeakyBucket, RateLimiter, RateLimiterError, TokenBucket};

// Re-export errors
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Rate limiter error: {0}")]
    RateLimiter(#[from] RateLimiterError),

    #[error("Circuit breaker error: {0}")]
    CircuitBreaker(#[from] CircuitBreakerError),

    #[error("Bulkhead error: partition exhausted")]
    BulkheadExhausted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let mut bucket = TokenBucket::new(10, 5);
        assert!(bucket.try_acquire());
    }

    #[tokio::test]
    async fn test_circuit_breaker_initial_state() {
        let cb = CircuitBreaker::new(5, std::time::Duration::from_secs(60));
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_bulkhead_partitions() {
        let bulkhead = Bulkhead::new(3, 10);
        let _guard = bulkhead.try_acquire(0).await.unwrap();
    }
}
