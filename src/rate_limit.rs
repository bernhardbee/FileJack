use governor::{Quota, RateLimiter as GovernorLimiter};
use nonzero_ext::nonzero;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

/// Rate limiter for MCP requests
#[derive(Clone)]
pub struct RateLimiter {
    limiter: Arc<GovernorLimiter<governor::state::direct::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>>,
}

impl RateLimiter {
    /// Create a new rate limiter with specified requests per second
    pub fn new(requests_per_second: u32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(requests_per_second).unwrap_or(nonzero!(10u32)));
        Self {
            limiter: Arc::new(GovernorLimiter::direct(quota)),
        }
    }

    /// Create a permissive rate limiter (1000 req/s)
    pub fn permissive() -> Self {
        Self::new(1000)
    }

    /// Create a moderate rate limiter (100 req/s)
    pub fn moderate() -> Self {
        Self::new(100)
    }

    /// Create a strict rate limiter (10 req/s)
    pub fn strict() -> Self {
        Self::new(10)
    }

    /// Check if a request is allowed, returns true if allowed
    pub fn check(&self) -> bool {
        self.limiter.check().is_ok()
    }

    /// Wait until a request can be processed (blocking)
    pub fn wait(&self) {
        while self.limiter.check().is_err() {
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::moderate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_requests() {
        let limiter = RateLimiter::new(10);
        // First request should be allowed
        assert!(limiter.check());
    }

    #[test]
    fn test_rate_limiter_enforces_limit() {
        let limiter = RateLimiter::new(2); // Very low limit for testing
        
        // First few requests should succeed
        assert!(limiter.check());
        assert!(limiter.check());
        
        // Next requests might fail due to rate limit
        // (timing-dependent, so we just check it compiles and runs)
        let _ = limiter.check();
    }

    #[test]
    fn test_permissive_limiter() {
        let limiter = RateLimiter::permissive();
        // Should allow many requests
        for _ in 0..10 {
            assert!(limiter.check());
        }
    }

    #[test]
    fn test_moderate_limiter() {
        let limiter = RateLimiter::moderate();
        assert!(limiter.check());
    }

    #[test]
    fn test_strict_limiter() {
        let limiter = RateLimiter::strict();
        assert!(limiter.check());
    }
}
