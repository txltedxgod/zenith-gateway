use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::Arc;
use zenith_gateway::config::RouteConfig;
use zenith_gateway::limiter::TokenBucket;
use zenith_gateway::router::Router;
use zenith_gateway::waf::WafEngine;

/// Benchmarking WAF regex inspection throughput and latencies
fn bench_waf(c: &mut Criterion) {
    let waf = WafEngine::new();
    let empty_blocked: Vec<String> = Vec::new();

    let mut group = c.benchmark_group("waf_inspection");

    // 1. Clean Request (Baseline Happy Path)
    group.bench_function("clean_request", |b| {
        b.iter(|| {
            waf.inspect(
                black_box("/api/v1/users/profile"),
                black_box("page=1&limit=20&sort=asc"),
                black_box(r#"{"user_id":"12345","query":"software engineer"}"#),
                black_box("192.168.1.1"),
                black_box(&empty_blocked),
            )
        })
    });

    // 2. SQL Injection Attack Detection
    group.bench_function("sqli_detection", |b| {
        b.iter(|| {
            waf.inspect(
                black_box("/api/v1/products"),
                black_box("id=1' OR 1=1 --"),
                black_box(""),
                black_box("192.168.1.1"),
                black_box(&empty_blocked),
            )
        })
    });

    // 3. Cross-Site Scripting (XSS) Detection
    group.bench_function("xss_detection", |b| {
        b.iter(|| {
            waf.inspect(
                black_box("/api/v1/comments"),
                black_box(""),
                black_box("<script>alert(document.cookie)</script>"),
                black_box("192.168.1.1"),
                black_box(&empty_blocked),
            )
        })
    });

    // 4. Path Traversal Attack Detection
    group.bench_function("path_traversal_detection", |b| {
        b.iter(|| {
            waf.inspect(
                black_box("/api/static/../../../../etc/passwd"),
                black_box(""),
                black_box(""),
                black_box("192.168.1.1"),
                black_box(&empty_blocked),
            )
        })
    });

    group.finish();
}

/// Benchmarking Router path matching and prefix routing
fn bench_router(c: &mut Criterion) {
    let routes = vec![
        RouteConfig {
            id: "users-service".to_string(),
            path_prefix: "/api/users".to_string(),
            upstream_url: "http://127.0.0.1:8081".to_string(),
            methods: Some(vec!["GET".to_string(), "POST".to_string()]),
            strip_prefix: Some(true),
            auth_required: Some(false),
            rate_limit_tier: Some("standard".to_string()),
            timeout_ms: Some(5000),
        },
        RouteConfig {
            id: "orders-service".to_string(),
            path_prefix: "/api/orders".to_string(),
            upstream_url: "http://127.0.0.1:8082".to_string(),
            methods: Some(vec![
                "GET".to_string(),
                "POST".to_string(),
                "DELETE".to_string(),
            ]),
            strip_prefix: Some(true),
            auth_required: Some(true),
            rate_limit_tier: Some("premium".to_string()),
            timeout_ms: Some(5000),
        },
        RouteConfig {
            id: "telemetry-service".to_string(),
            path_prefix: "/telemetry".to_string(),
            upstream_url: "http://127.0.0.1:8083".to_string(),
            methods: None,
            strip_prefix: None,
            auth_required: Some(false),
            rate_limit_tier: None,
            timeout_ms: Some(2000),
        },
    ];

    let router = Router::new(routes);

    let mut group = c.benchmark_group("router_lookup");

    group.bench_function("route_match_hit", |b| {
        b.iter(|| router.match_route(black_box("/api/users/12345/profile"), black_box("GET")))
    });

    group.bench_function("route_match_miss", |b| {
        b.iter(|| {
            router.match_route(
                black_box("/api/unknown/unregistered/route"),
                black_box("GET"),
            )
        })
    });

    group.finish();
}

/// Benchmarking Atomic Token Bucket Rate Limiting under async contention
fn bench_rate_limiter(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap();

    let bucket = Arc::new(TokenBucket::new(1_000_000, 1_000_000));

    let mut group = c.benchmark_group("rate_limiter");

    group.bench_function("token_bucket_acquire", |b| {
        b.to_async(&rt).iter(|| {
            let b = bucket.clone();
            async move {
                black_box(b.check_and_consume(1).await);
            }
        })
    });

    group.finish();
}

criterion_group!(benches, bench_waf, bench_router, bench_rate_limiter);
criterion_main!(benches);
