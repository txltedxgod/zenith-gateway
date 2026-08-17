#[cfg(test)]
mod tests {
    use regex::Regex;

    #[test]
    fn test_waf_sqli_patterns() {
        let sqli_regex = Regex::new(r"(?i)(\b(UNION(\s+ALL)?|SELECT|INSERT|UPDATE|DELETE|DROP|ALTER|CREATE|EXEC)\b|--|\bOR\b\s+\d+=\d+|\bAND\b\s+\d+=\d+)").unwrap();
        
        assert!(sqli_regex.is_match("SELECT * FROM users WHERE id = 1"));
        assert!(sqli_regex.is_match("' UNION SELECT username, password FROM admin --"));
        assert!(sqli_regex.is_match("1 OR 1=1"));
        assert!(!sqli_regex.is_match("/api/v1/users/profile"));
    }

    #[test]
    fn test_waf_xss_patterns() {
        let xss_regex = Regex::new(r"(?i)(<script[\s>]|javascript:|onload=|onerror=|eval\(|<iframe[\s>])").unwrap();

        assert!(xss_regex.is_match("<script>alert(1)</script>"));
        assert!(xss_regex.is_match("<img src=x onerror=alert(1)>"));
        assert!(xss_regex.is_match("javascript:alert(document.cookie)"));
        assert!(!xss_regex.is_match("{\"name\":\"Alice\",\"age\":30}"));
    }

    #[test]
    fn test_waf_path_traversal() {
        let traversal_regex = Regex::new(r"(\.\./|\.\.\\|%2e%2e%2f|%2e%2e/|\.\.%2f)").unwrap();

        assert!(traversal_regex.is_match("/etc/passwd/../../../secret"));
        assert!(traversal_regex.is_match("/api/files/%2e%2e/config.json"));
        assert!(!traversal_regex.is_match("/api/v1/static/image.png"));
    }
}
