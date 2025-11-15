use crate::{state::AppState, util};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use rusty_relay_messages::RelayMessage;
use std::sync::Arc;

pub async fn webhook_handler(
    state: State<Arc<AppState>>,
    headers: HeaderMap,
    method: axum::http::Method,
    Path(client_id): Path<String>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    tracing::info!("📩 webhook ({method}) received for client with id: {client_id}");

    for (k, v) in headers.iter() {
        if let Ok(value) = v.to_str() {
            tracing::debug!("header: {} = {}", k, value);
        }
    }
    if let Ok(b) = String::from_utf8(body.to_vec()) {
        tracing::debug!("body: {}", b);
    }

    if let Some(sender) = state.get_client(&client_id).await {
        let _ = sender.send(RelayMessage::Webhook {
            method: method.to_string(),
            body: body.to_vec(),
            headers: util::into_hashmap(headers),
        });
    } else {
        return (
            StatusCode::BAD_REQUEST,
            format!("Client id is unknown: {}", client_id),
        )
            .into_response();
    }

    StatusCode::OK.into_response()
}
