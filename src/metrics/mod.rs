use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub struct MetricsRegistry {
    requests_total: AtomicU64,
    waf_blocks_total: AtomicU64,
    rate_limited_total: AtomicU64,
    start_time: Instant,
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            requests_total: AtomicU64::new(0),
            waf_blocks_total: AtomicU64::new(0),
            rate_limited_total: AtomicU64::new(0),
            start_time: Instant::now(),
        }
    }

    pub fn inc_requests(&self) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_waf_blocks(&self) {
        self.waf_blocks_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_rate_limited(&self) {
        self.rate_limited_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn render_prometheus(&self) -> String {
        let uptime = self.start_time.elapsed().as_secs();
        format!(
            "# HELP zenith_requests_total Total number of HTTP requests processed\n\
             # TYPE zenith_requests_total counter\n\
             zenith_requests_total {}\n\
             # HELP zenith_waf_blocks_total Total number of requests blocked by WAF\n\
             # TYPE zenith_waf_blocks_total counter\n\
             zenith_waf_blocks_total {}\n\
             # HELP zenith_rate_limited_total Total number of requests rejected by Rate Limiter\n\
             # TYPE zenith_rate_limited_total counter\n\
             zenith_rate_limited_total {}\n\
             # HELP zenith_uptime_seconds Gateway uptime in seconds\n\
             # TYPE zenith_uptime_seconds gauge\n\
             zenith_uptime_seconds {}\n",
            self.requests_total.load(Ordering::Relaxed),
            self.waf_blocks_total.load(Ordering::Relaxed),
            self.rate_limited_total.load(Ordering::Relaxed),
            uptime
        )
    }
}
