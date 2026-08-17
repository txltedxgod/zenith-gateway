use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub server: ServerConfig,
    pub routes: Vec<RouteConfig>,
    pub rate_limits: Option<HashMap<String, RateLimitSpec>>,
    pub waf: Option<WafConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub admin_port: Option<u16>,
    pub workers: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    pub id: String,
    pub path_prefix: String,
    pub upstream_url: String,
    pub methods: Option<Vec<String>>,
    pub strip_prefix: Option<bool>,
    pub auth_required: Option<bool>,
    pub rate_limit_tier: Option<String>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitSpec {
    pub rate_per_second: u64,
    pub burst_capacity: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WafConfig {
    pub enabled: bool,
    pub block_sqli: bool,
    pub block_xss: bool,
    pub block_path_traversal: bool,
    pub blocked_ips: Option<Vec<String>>,
}

impl GatewayConfig {
    pub fn from_yaml(yaml_str: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml_str)
    }
}
