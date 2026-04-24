//! Sentinel patterns: rate limiting, circuit breaker, bulkhead.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
}

pub struct Sentinel {
    config: RateLimitConfig,
}

impl Sentinel {
    pub fn new(config: RateLimitConfig) -> Self {
        Self { config }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config() {
        let cfg = RateLimitConfig {
            requests_per_second: 100,
            burst_size: 10,
        };
        assert_eq!(cfg.requests_per_second, 100);
    }
}
