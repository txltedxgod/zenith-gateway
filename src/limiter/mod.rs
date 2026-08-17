use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Token bucket algorithm for high-performance lock-free rate limiting.
#[derive(Debug)]
pub struct TokenBucket {
    rate_per_sec: u64,
    capacity: u64,
    tokens: AtomicU64,
    last_replenished: Arc<RwLock<Instant>>,
}

impl TokenBucket {
    pub fn new(rate_per_sec: u64, capacity: u64) -> Self {
        Self {
            rate_per_sec,
            capacity,
            tokens: AtomicU64::new(capacity),
            last_replenished: Arc::new(RwLock::new(Instant::now())),
        }
    }

    pub async fn check_and_consume(&self, count: u64) -> bool {
        self.replenish().await;

        loop {
            let current = self.tokens.load(Ordering::Relaxed);
            if current < count {
                return false; // Rate limit exceeded
            }
            if self
                .tokens
                .compare_exchange_weak(
                    current,
                    current - count,
                    Ordering::SeqCst,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                return true;
            }
        }
    }

    async fn replenish(&self) {
        let mut last = self.last_replenished.write().await;
        let now = Instant::now();
        let elapsed_secs = now.duration_since(*last).as_secs_f64();

        if elapsed_secs >= 0.1 {
            let new_tokens = (elapsed_secs * self.rate_per_sec as f64) as u64;
            if new_tokens > 0 {
                let current = self.tokens.load(Ordering::Relaxed);
                let replenished = (current + new_tokens).min(self.capacity);
                self.tokens.store(replenished, Ordering::Relaxed);
                *last = now;
            }
        }
    }
}

#[derive(Default)]
pub struct RateLimiterManager {
    buckets: Arc<RwLock<HashMap<String, Arc<TokenBucket>>>>,
}

impl RateLimiterManager {
    pub fn new() -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn acquire(&self, client_key: &str, rate: u64, burst: u64) -> bool {
        let bucket = {
            let read_lock = self.buckets.read().await;
            if let Some(b) = read_lock.get(client_key) {
                b.clone()
            } else {
                drop(read_lock);
                let mut write_lock = self.buckets.write().await;
                write_lock
                    .entry(client_key.to_string())
                    .or_insert_with(|| Arc::new(TokenBucket::new(rate, burst)))
                    .clone()
            }
        };

        bucket.check_and_consume(1).await
    }
}
