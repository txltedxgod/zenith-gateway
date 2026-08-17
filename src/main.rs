mod auth;
mod config;
mod limiter;
mod metrics;
mod router;
mod waf;

use config::GatewayConfig;
use limiter::RateLimiterManager;
use metrics::MetricsRegistry;
use router::Router;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::{error, info};
use waf::{WafDecision, WafEngine};

const BANNER: &str = r#"
  ______            _ _   _        _____       _                            
 |___  /           (_) | | |      / ____|     | |                           
    / / ___ _ __  _ _| |_| |__   | |  __  __ _| |_ _____      ____ _ _   _ 
   / / / _ \ '_ \| | | __| '_ \  | | |_ |/ _` | __/ _ \ \ /\ / / _` | | | |
  / /_|  __/ | | | | | |_| | | | | |__| | (_| | ||  __/\ V  V / (_| | |_| |
 /_____\___|_| |_|_|_|\__|_| |_|  \_____|\__,_|\__\___| \_/\_/ \__,_|\__, |
 High-Performance Cloud-Native API Gateway & WAF                       __/ |
                                                                      |___/ 
"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    println!("{}", BANNER);

    let default_config = r#"
server:
  host: "0.0.0.0"
  port: 8000
routes:
  - id: "users-service"
    path_prefix: "/api/users"
    upstream_url: "http://127.0.0.1:8081"
    auth_required: false
  - id: "orders-service"
    path_prefix: "/api/orders"
    upstream_url: "http://127.0.0.1:8082"
    auth_required: true
waf:
  enabled: true
  block_sqli: true
  block_xss: true
  block_path_traversal: true
"#;

    let config = GatewayConfig::from_yaml(default_config)?;
    let bind_addr = format!("{}:{}", config.server.host, config.server.port);

    let router = Arc::new(Router::new(config.routes.clone()));
    let waf = Arc::new(WafEngine::new());
    let limiter = Arc::new(RateLimiterManager::new());
    let metrics = Arc::new(MetricsRegistry::new());

    let listener = TcpListener::bind(&bind_addr).await?;
    info!("Zenith Gateway listening on http://{}", bind_addr);

    loop {
        let (mut socket, client_addr) = listener.accept().await?;
        let router = router.clone();
        let waf = waf.clone();
        let limiter = limiter.clone();
        let metrics = metrics.clone();

        tokio::spawn(async move {
            let mut buf = [0u8; 4096];
            let n = match socket.read(&mut buf).await {
                Ok(n) if n > 0 => n,
                _ => return,
            };

            let req_str = String::from_utf8_lossy(&buf[..n]);
            let lines: Vec<&str> = req_str.lines().collect();
            if lines.is_empty() {
                return;
            }

            metrics.inc_requests();
            let first_line = lines[0];
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            if parts.len() < 2 {
                return;
            }

            let method = parts[0];
            let path = parts[1];
            let client_ip = client_addr.ip().to_string();

            // Admin endpoints
            if path == "/metrics" {
                let body = metrics.render_prometheus();
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = socket.write_all(resp.as_bytes()).await;
                return;
            }

            if path == "/health" {
                let body = r#"{"status":"healthy","gateway":"zenith-rs"}"#;
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = socket.write_all(resp.as_bytes()).await;
                return;
            }

            // 1. WAF Inspection
            let decision = waf.inspect(path, "", &req_str, &client_ip, &[]);
            if let WafDecision::Block(reason) = decision {
                metrics.inc_waf_blocks();
                let body = format!(
                    r#"{{"error":"Forbidden by Zenith WAF","reason":"{}"}}"#,
                    reason
                );
                let resp = format!(
                    "HTTP/1.1 403 Forbidden\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = socket.write_all(resp.as_bytes()).await;
                return;
            }

            // 2. Rate Limiting Check
            if !limiter.acquire(&client_ip, 100, 200).await {
                metrics.inc_rate_limited();
                let body = r#"{"error":"Too Many Requests","message":"Rate limit exceeded"}"#;
                let resp = format!(
                    "HTTP/1.1 429 Too Many Requests\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = socket.write_all(resp.as_bytes()).await;
                return;
            }

            // 3. Routing
            if let Some(route) = router.match_route(path, method) {
                let body = format!(
                    r#"{{"proxy":"zenith-gateway","status":"forwarded","route_id":"{}","upstream":"{}"}}"#,
                    route.id, route.upstream_url
                );
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = socket.write_all(resp.as_bytes()).await;
            } else {
                let body = r#"{"error":"Not Found","message":"No matching route"}"#;
                let resp = format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = socket.write_all(resp.as_bytes()).await;
            }
        });
    }
}
