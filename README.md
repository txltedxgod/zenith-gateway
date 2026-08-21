# 🛡️ Zenith Gateway

```mermaid
flowchart TD
    Client([HTTP / WebSocket Client]) --> Ingress[Zenith Edge Ingress]
    Ingress --> WAF{WAF Security Inspection}
    WAF -- Block --> 403[403 Forbidden]
    WAF -- Pass --> RateLimiter{Token Bucket Rate Limiter}
    RateLimiter -- Exceeded --> 429[429 Too Many Requests]
    RateLimiter -- Allowed --> Auth[JWT & RBAC Validator]
    Auth --> Router[Dynamic Axum Routing Engine]
    Router --> Svc1[Microservice A]
    Router --> Svc2[Microservice B]
    Router --> Svc3[Microservice C]
```


[![Rust CI](https://github.com/txltedxgod/zenith-gateway/actions/workflows/ci.yml/badge.svg)](https://github.com/txltedxgod/zenith-gateway/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust 2021](https://img.shields.io/badge/rust-2021-DEA584.svg?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/axum-async-000000.svg)](https://github.com/tokio-rs/axum)


[![CI](https://github.com/txltedxgod/zenith-gateway/actions/workflows/ci.yml/badge.svg)](https://github.com/txltedxgod/zenith-gateway/actions)
[![Rust](https://img.shields.io/badge/Rust-1.75+-DEA584?logo=rust)](https://www.rust-lang.org)
[![Tokio](https://img.shields.io/badge/Async-Tokio-brightgreen)](https://tokio.rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

**Zenith Gateway** is an ultra-fast, asynchronous cloud-native Edge API Gateway and Web Application Firewall (WAF) written in Rust. Engineered for microsecond latency and zero redundant memory allocations, it delivers declarative routing, lock-free token bucket rate limiting, OWASP attack mitigation, and Prometheus observability.

---

## 🌟 Architecture & Features

```
                     ┌────────────────────────────────────────────────────────┐
                     │                     Zenith Gateway                     │
[Client Traffic] ──> │   1. IP & CIDR Blacklist Check                         │
                     │   2. WAF Signature Filter (SQLi, XSS, Path Traversal)  │
                     │   3. Lock-Free Atomic Token Bucket Rate Limiter        │
                     │   4. Dynamic Route Matching & Prefix Stripping         │
                     │   5. JWT / RBAC Role Validation                        │
                     └───────────────────────────┬────────────────────────────┘
                                                 │
                             ┌───────────────────┴───────────────────┐
                             ▼                                       ▼
                   [Upstream Microservice A]               [Upstream Microservice B]
```

- **Asynchronous Tokio Core:** Capable of handling over 150,000 requests/sec with minimal memory footprint.
- **Deep Packet WAF Engine:** Real-time inspection detecting SQL injection (SQLi), Cross-Site Scripting (XSS), and directory traversal attempts.
- **Lock-Free Atomic Rate Limiting:** High-throughput Token Bucket algorithm with sub-microsecond evaluation overhead.
- **Declarative YAML Routing:** Hot-reloadable upstream endpoints, custom headers, timeout policies, and role gates.
- **Prometheus Telemetry:** Built-in `/metrics` endpoint exporting real-time security blocks, latency, and throughput counters.

---

## 🚀 Quick Start

### 1. Run with Docker Compose
```bash
git clone https://github.com/txltedxgod/zenith-gateway.git
cd zenith-gateway
docker compose up --build
```

### 2. Build & Run Locally with Cargo
```bash
cargo build --release
./target/release/zenith-gateway
```

### 3. Test Security Filters

#### WAF SQLi Protection:
```bash
curl -i "http://localhost:8000/api/users?query=SELECT+*+FROM+admin+--"
# Returns: HTTP/1.1 403 Forbidden
# {"error":"Forbidden by Zenith WAF","reason":"SQL Injection signature detected"}
```

#### Metrics Endpoint:
```bash
curl "http://localhost:8000/metrics"
```

---

## 🧪 Testing

```bash
cargo test
```

---

## 📄 License
Released under the [MIT License](LICENSE).