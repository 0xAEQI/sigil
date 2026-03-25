use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::server::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

/// Create a JWT token.
pub fn create_token(
    secret: &str,
    expiry_hours: u64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: "operator".to_string(),
        iat: now,
        exp: now + (expiry_hours * 3600) as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Validate a JWT token and return claims.
pub fn validate_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}

/// Extract Bearer token from Authorization header.
fn extract_bearer(req: &Request) -> Option<&str> {
    req.headers()
        .get("authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

/// Axum middleware that validates JWT using auth_secret from AppState.
pub async fn require_auth(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let secret = state.auth_secret.as_deref().unwrap_or("");

    let Some(token) = extract_bearer(&req) else {
        return (StatusCode::UNAUTHORIZED, "missing authorization header").into_response();
    };

    match validate_token(token, secret) {
        Ok(_) => next.run(req).await,
        Err(_) => (StatusCode::UNAUTHORIZED, "invalid or expired token").into_response(),
    }
}
