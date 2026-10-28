#[cfg(test)]
mod tests {
    use zenith_gateway::auth::jwt::AuthValidator;
    use zenith_gateway::config::RouteConfig;
    use zenith_gateway::router::Router;
    use zenith_gateway::waf::{WafDecision, WafEngine};

    #[test]
    fn test_waf_sqli_detection() {
        let waf = WafEngine::new();
        let blocked_ips = vec![];

        let decision = waf.inspect(
            "/api/users",
            "id=1' OR 1=1 --",
            "",
            "127.0.0.1",
            &blocked_ips,
        );
        assert!(matches!(decision, WafDecision::Block(_)));

        let clean = waf.inspect(
            "/api/users/123",
            "tab=activity",
            r#"{"role":"developer"}"#,
            "127.0.0.1",
            &blocked_ips,
        );
        assert_eq!(clean, WafDecision::Allow);
    }

    #[test]
    fn test_waf_xss_detection() {
        let waf = WafEngine::new();
        let blocked_ips = vec![];

        let decision = waf.inspect(
            "/api/comments",
            "",
            "<script>alert('xss')</script>",
            "127.0.0.1",
            &blocked_ips,
        );
        assert!(matches!(decision, WafDecision::Block(_)));
    }

    #[test]
    fn test_waf_path_traversal() {
        let waf = WafEngine::new();
        let blocked_ips = vec![];

        let decision = waf.inspect(
            "/api/static/../../secret.env",
            "",
            "",
            "127.0.0.1",
            &blocked_ips,
        );
        assert!(matches!(decision, WafDecision::Block(_)));
    }

    #[test]
    fn test_router_matching() {
        let routes = vec![RouteConfig {
            id: "users".to_string(),
            path_prefix: "/api/users".to_string(),
            upstream_url: "http://127.0.0.1:8081".to_string(),
            methods: Some(vec!["GET".to_string(), "POST".to_string()]),
            strip_prefix: None,
            auth_required: None,
            rate_limit_tier: None,
            timeout_ms: None,
        }];
        let router = Router::new(routes);

        assert!(router.match_route("/api/users/profile", "GET").is_some());
        assert!(router.match_route("/api/users/profile", "DELETE").is_none());
        assert!(router.match_route("/api/orders", "GET").is_none());
    }

    #[test]
    fn test_auth_validator() {
        assert!(AuthValidator::validate_header(None).is_err());
        assert!(AuthValidator::validate_header(Some("Basic 123")).is_err());
        assert!(AuthValidator::validate_header(Some("Bearer ")).is_err());
        assert!(AuthValidator::validate_header(Some("Bearer eyJhbGciOi...")).is_ok());
    }
}
