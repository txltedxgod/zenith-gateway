use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub roles: Option<Vec<String>>,
}

pub struct AuthValidator;

impl AuthValidator {
    pub fn validate_header(auth_header: Option<&str>) -> Result<Claims, String> {
        let header = auth_header.ok_or_else(|| "Missing Authorization header".to_string())?;
        
        if !header.starts_with("Bearer ") {
            return Err("Invalid token format, expected 'Bearer <token>'".to_string());
        }

        let token = &header[7..];
        if token.is_empty() {
            return Err("Empty bearer token".to_string());
        }

        // Mock token validation / decode claims for demo & tests
        Ok(Claims {
            sub: "user-12345".to_string(),
            exp: 9999999999,
            roles: Some(vec!["admin".to_string(), "user".to_string()]),
        })
    }
}
