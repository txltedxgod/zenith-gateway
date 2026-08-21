// Zenith Gateway - Rate Limiter Integration Tests

#[cfg(test)]
mod tests {
    #[derive(Debug, Clone)]
    pub struct TokenBucket {
        capacity: u64,
        available: u64,
    }

    impl TokenBucket {
        pub fn new(capacity: u64) -> Self {
            Self { capacity, available: capacity }
        }

        pub fn try_acquire(&mut self, tokens: u64) -> bool {
            if self.available >= tokens {
                self.available -= tokens;
                true
            } else {
                false
            }
        }
    }

    #[test]
    fn test_token_bucket_acquire() {
        let mut bucket = TokenBucket::new(10);
        assert!(bucket.try_acquire(5));
        assert!(bucket.try_acquire(5));
        assert!(!bucket.try_acquire(1));
    }
}
