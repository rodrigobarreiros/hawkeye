use std::time::Duration;

use crate::domain::errors::{DomainError, Result};

/// Backoff configuration for reconnection attempts
#[derive(Debug, Clone, PartialEq)]
pub struct BackoffPolicy {
    initial_delay: Duration,
    max_delay: Duration,
    multiplier: f64,
}

impl BackoffPolicy {
    pub fn new(initial_delay: Duration, max_delay: Duration, multiplier: f64) -> Result<Self> {
        if multiplier <= 1.0 {
            return Err(DomainError::InvalidBackoffMultiplier);
        }

        Ok(Self {
            initial_delay,
            max_delay,
            multiplier,
        })
    }

    pub fn initial_delay(&self) -> Duration {
        self.initial_delay
    }

    pub fn max_delay(&self) -> Duration {
        self.max_delay
    }

    pub fn multiplier(&self) -> f64 {
        self.multiplier
    }

    /// Calculate the next backoff delay based on current delay
    pub fn next_delay(&self, current: Duration) -> Duration {
        let next = Duration::from_secs_f64(current.as_secs_f64() * self.multiplier);
        next.min(self.max_delay)
    }
}

impl Default for BackoffPolicy {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = BackoffPolicy::default();
        assert_eq!(policy.initial_delay(), Duration::from_secs(1));
        assert_eq!(policy.max_delay(), Duration::from_secs(30));
        assert_eq!(policy.multiplier(), 2.0);
    }

    #[test]
    fn test_next_delay_doubles() {
        let policy = BackoffPolicy::default();
        let current = Duration::from_secs(1);
        let next = policy.next_delay(current);
        assert_eq!(next, Duration::from_secs(2));
    }

    #[test]
    fn test_next_delay_caps_at_max() {
        let policy = BackoffPolicy::default();
        let current = Duration::from_secs(20);
        let next = policy.next_delay(current);
        assert_eq!(next, Duration::from_secs(30)); // Capped at max
    }

    #[test]
    fn test_rejects_invalid_multiplier() {
        let result = BackoffPolicy::new(
            Duration::from_secs(1),
            Duration::from_secs(30),
            1.0, // Invalid: must be > 1.0
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_accepts_valid_multiplier() {
        let result = BackoffPolicy::new(
            Duration::from_secs(1),
            Duration::from_secs(60),
            1.5,
        );
        assert!(result.is_ok());
    }
}
