//! # phenotype-sentinel
//!
//! Circuit breaker implementation for fault tolerance.

use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CircuitBreakerError {
    #[error("Circuit breaker is open")]
    Open,

    #[error("Circuit breaker is half-open, request not allowed")]
    HalfOpen,
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation, requests pass through
    Closed,
    /// Circuit is open, requests are blocked
    Open,
    /// Testing if service has recovered
    HalfOpen,
}

/// Circuit breaker for fault tolerance
///
/// Opens the circuit when failure threshold is reached,
/// preventing cascading failures.
#[derive(Debug)]
pub struct CircuitBreaker {
    failure_threshold: usize,
    recovery_timeout: Duration,
    failure_count: usize,
    last_failure: Option<Instant>,
    state: CircuitState,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    ///
    /// - `failure_threshold`: Number of failures before opening
    /// - `recovery_timeout`: Time to wait before trying recovery
    pub fn new(failure_threshold: usize, recovery_timeout: Duration) -> Self {
        Self {
            failure_threshold,
            recovery_timeout,
            failure_count: 0,
            last_failure: None,
            state: CircuitState::Closed,
        }
    }

    /// Get current circuit state
    pub fn state(&self) -> CircuitState {
        self.state
    }

    /// Check if requests are allowed
    pub fn is_allowed(&self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if we should transition to half-open
                self.last_failure
                    .map(|last| last.elapsed() >= self.recovery_timeout)
                    .unwrap_or(false)
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful request
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                // Transition to closed on successful request
                self.state = CircuitState::Closed;
                self.failure_count = 0;
            }
            CircuitState::Open => {
                // Should not receive success in open state
            }
        }
    }

    /// Record a failed request
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Instant::now());

        match self.state {
            CircuitState::Closed => {
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open;
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open goes back to open
                self.state = CircuitState::Open;
            }
            CircuitState::Open => {
                // Already open, stay open
            }
        }
    }

    /// Force the circuit to a specific state
    pub fn force_state(&mut self, state: CircuitState) {
        self.state = state;
        if state == CircuitState::Closed {
            self.failure_count = 0;
        }
    }

    /// Reset the circuit breaker
    pub fn reset(&mut self) {
        self.state = CircuitState::Closed;
        self.failure_count = 0;
        self.last_failure = None;
    }

    /// Execute a function with circuit breaker protection
    pub fn execute<F, T, E>(&mut self, f: F) -> Result<T, CircuitBreakerError>
    where
        F: FnOnce() -> Result<T, E>,
    {
        if !self.is_allowed() {
            return Err(CircuitBreakerError::Open);
        }

        match self.state {
            CircuitState::HalfOpen => match f() {
                Ok(result) => {
                    self.record_success();
                    Ok(result)
                }
                Err(_) => {
                    self.record_failure();
                    Err(CircuitBreakerError::HalfOpen)
                }
            },
            _ => match f() {
                Ok(result) => {
                    self.record_success();
                    Ok(result)
                }
                Err(_) => {
                    self.record_failure();
                    Err(CircuitBreakerError::Open)
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_initial_state() {
        let cb = CircuitBreaker::new(5, Duration::from_secs(60));
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_opens_on_threshold() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(60));

        for _ in 0..3 {
            cb.record_failure();
        }

        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn test_circuit_breaker_success_resets() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(60));

        cb.record_failure();
        cb.record_failure();
        cb.record_success();

        assert_eq!(cb.failure_count, 0);
    }
}
