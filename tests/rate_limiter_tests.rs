use std::sync::Arc;
use zenith_gateway::limiter::{RateLimiterManager, TokenBucket};

#[tokio::test]
async fn test_token_bucket_replenish_and_consume() {
    let bucket = TokenBucket::new(100, 10);

    // Initial consume within capacity
    assert!(bucket.check_and_consume(5).await);
    assert!(bucket.check_and_consume(5).await);

    // Exceeded capacity
    assert!(!bucket.check_and_consume(1).await);
}

#[tokio::test]
async fn test_rate_limiter_manager_tiers() {
    let manager = RateLimiterManager::new();

    // Standard client check
    let allowed = manager.check_rate_limit("client-ip-123", "standard").await;
    assert!(allowed);

    // Unknown tier fallback
    let fallback = manager.check_rate_limit("client-ip-456", "non_existent_tier").await;
    assert!(fallback);
}

#[tokio::test]
async fn test_high_concurrency_token_acquisition() {
    let bucket = Arc::new(TokenBucket::new(100_000, 50));
    let mut handles = Vec::new();

    for _ in 0..50 {
        let b = bucket.clone();
        handles.push(tokio::spawn(async move {
            b.check_and_consume(1).await
        }));
    }

    let mut success_count = 0;
    for h in handles {
        if h.await.unwrap() {
            success_count += 1;
        }
    }

    assert_eq!(success_count, 50);
}
