use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use rusty_relay_messages::RelayMessage;

use crate::state::AppState;

pub async fn webhook_handler(
    state: State<Arc<AppState>>,
    Path(client_id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    tracing::info!("📩 webhook received for client with id: {client_id}");
    tracing::debug!("{}", payload);

    if let Some(sender) = state.get_client(&client_id).await {
        let _ = sender.send(RelayMessage::Webhook {
            payload: payload.to_string(),
        });
    }

    StatusCode::OK
}
