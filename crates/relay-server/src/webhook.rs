use crate::{error::HttpError, state::AppState, util};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use rusty_relay_messages::RelayMessage;
use std::sync::Arc;
use tracing::info;

#[tracing::instrument(skip(state))]
pub async fn webhook_handler(
    state: State<Arc<AppState>>,
    headers: HeaderMap,
    method: axum::http::Method,
    Path(client_id): Path<String>,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    info!("📩 webhook received");

    if let Some(sender) = state.get_client(&client_id).await {
        let _ = sender.send(RelayMessage::Webhook {
            method: method.to_string(),
            body: body.to_vec(),
            headers: util::into_hashmap(headers),
        });
    } else {
        return HttpError::BadRequest(format!("Client id is unknown: {}", client_id))
            .into_response();
    }

    StatusCode::OK.into_response()
}
