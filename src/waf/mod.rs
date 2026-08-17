use regex::Regex;
use std::sync::Arc;

pub struct WafEngine {
    sqli_regex: Arc<Regex>,
    xss_regex: Arc<Regex>,
    path_traversal_regex: Arc<Regex>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum WafDecision {
    Allow,
    Block(String),
}

impl Default for WafEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl WafEngine {
    pub fn new() -> Self {
        Self {
            sqli_regex: Arc::new(
                Regex::new(r"(?i)(\b(UNION(\s+ALL)?|SELECT|INSERT|UPDATE|DELETE|DROP|ALTER|CREATE|EXEC)\b|--|\bOR\b\s+\d+=\d+|\bAND\b\s+\d+=\d+)").unwrap()
            ),
            xss_regex: Arc::new(
                Regex::new(r"(?i)(<script[\s>]|javascript:|onload=|onerror=|eval\(|<iframe[\s>])").unwrap()
            ),
            path_traversal_regex: Arc::new(
                Regex::new(r"(\.\./|\.\.\\|%2e%2e%2f|%2e%2e/|\.\.%2f)").unwrap()
            ),
        }
    }

    pub fn inspect(&self, path: &str, query: &str, body: &str, ip: &str, blocked_ips: &[String]) -> WafDecision {
        // 1. Check IP Blocklist
        if blocked_ips.iter().any(|b| b == ip) {
            return WafDecision::Block(format!("IP address {} is blacklisted", ip));
        }

        // 2. Path Traversal
        if self.path_traversal_regex.is_match(path) || self.path_traversal_regex.is_match(query) {
            return WafDecision::Block("Path traversal pattern detected".to_string());
        }

        // 3. SQL Injection
        if self.sqli_regex.is_match(path) || self.sqli_regex.is_match(query) || self.sqli_regex.is_match(body) {
            return WafDecision::Block("SQL Injection signature detected".to_string());
        }

        // 4. Cross-Site Scripting (XSS)
        if self.xss_regex.is_match(path) || self.xss_regex.is_match(query) || self.xss_regex.is_match(body) {
            return WafDecision::Block("Cross-Site Scripting (XSS) signature detected".to_string());
        }

        WafDecision::Allow
    }
}
