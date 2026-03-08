mod companies;

use axum::{Json, Router, middleware, routing::get};
use serde::Serialize;

use crate::auth::require_auth;
use crate::state::AppState;
use crate::ws;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// Top-level health route at `/health` (no auth).
pub fn health_router() -> Router<AppState> {
    Router::new().route("/health", get(health))
}

/// Protected API routes under `/api`.
pub fn api_router(secret: String) -> Router<AppState> {
    let protected = Router::new()
        .merge(companies::router())
        .route_layer(middleware::from_fn_with_state(secret, require_auth));

    // WebSocket: uses query-param auth (not middleware)
    let ws = Router::new().route("/companies/{company_id}/stream", get(ws::stream::handler));

    Router::new().merge(protected).merge(ws)
}
