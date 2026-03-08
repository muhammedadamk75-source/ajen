use axum::{
    extract::{Path, Query, State, WebSocketUpgrade, ws::WebSocket},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::info;

use crate::auth::WsAuthQuery;
use crate::state::AppState;

pub async fn handler(
    ws: WebSocketUpgrade,
    Path(company_id): Path<String>,
    Query(auth): Query<WsAuthQuery>,
    State(state): State<AppState>,
) -> Response {
    if auth.token != state.secret {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    ws.on_upgrade(move |socket| handle_socket(socket, company_id, state))
}

async fn handle_socket(mut socket: WebSocket, company_id: String, state: AppState) {
    info!(company_id = %company_id, "WebSocket connected");

    let mut rx = state.engine.event_bus.subscribe();

    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    Ok(event) if event.company_id == company_id => {
                        if let Ok(json) = serde_json::to_string(&event) {
                            if socket.send(axum::extract::ws::Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(_) => break,
                    _ => {}
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(_)) => {} // Ignore client messages
                    _ => break,
                }
            }
        }
    }

    info!(company_id = %company_id, "WebSocket disconnected");
}
