use axum::{
    Json,
    extract::{Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde::Deserialize;

/// Generate a cryptographically random secret: `ak_` + 32 hex chars.
pub fn generate_secret() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);
    format!("ak_{}", hex::encode(bytes))
}

/// Axum middleware that validates `Authorization: Bearer <token>` on every request.
/// State is the auth token string, provided via `from_fn_with_state`.
pub async fn require_auth(State(token): State<String>, request: Request, next: Next) -> Response {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let expected = format!("Bearer {}", token);

    match auth_header {
        Some(value) if value == expected => next.run(request).await,
        _ => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "Invalid or missing secret" })),
        )
            .into_response(),
    }
}

/// Query parameters for WebSocket authentication.
#[derive(Deserialize)]
pub struct WsAuthQuery {
    pub token: String,
}
