use crate::{error::HttpError, state::AppState, util::generate_id};
use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::HeaderMap,
    response::IntoResponse,
};
use rusty_relay_messages::RelayMessage;
use std::sync::Arc;
use tokio::time;
use tokio_stream::StreamExt;
use tracing::{debug, error, info};

#[tracing::instrument(skip(ws, state))]
pub async fn connect_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    state: State<Arc<AppState>>,
) -> impl IntoResponse {
    match headers.get("PRIVATE-TOKEN") {
        Some(token) => match token.to_str() {
            Ok(token) => {
                if token == state.connect_token() {
                    let client_id = generate_id(12);
                    info!(client_id, "👨 client connected");
                    ws.on_upgrade(move |socket| handle_ws(socket, client_id, state))
                } else {
                    debug!("❌ client provided invalid token");
                    HttpError::Unauthorized("Connection token is invalid".to_string())
                        .into_response()
                }
            }
            Err(_) => {
                debug!("❌ client provided invalid token format");
                HttpError::BadRequest("Connection token has invalid format".to_string())
                    .into_response()
            }
        },
        None => {
            debug!("❌ client did not provide token");
            HttpError::Unauthorized("Connection token is missing from header".to_string())
                .into_response()
        }
    }
}

#[tracing::instrument(skip(socket, state))]
async fn handle_ws(mut socket: WebSocket, client_id: String, state: State<Arc<AppState>>) {
    if let Ok(msg) = serde_json::to_string(&RelayMessage::ClientId(client_id.clone())) {
        if socket.send(Message::Text(msg.into())).await.is_err() {
            error!("failed to send message to client");
            return;
        }
    } else {
        error!("failed to serialize into JSON");
        return;
    }

    let mut rx_relay = state.add_client(&client_id).await;
    let mut ping_interval = time::interval(state.ping_interval());

    loop {
        tokio::select! {
            _ = ping_interval.tick() => {
                if socket.send(Message::Ping(Vec::new().into())).await.is_err() {
                    error!("failed to send ping to client");
                    break;
                }
            }
            Ok(relay_message) = rx_relay.recv() => {
                if let Ok(msg) = serde_json::to_string(&relay_message) {
                    if socket
                        .send(Message::Text(msg.into()))
                        .await
                        .is_err()
                    {
                        error!("failed to send message to client");
                        break;
                    }
                } else {
                    error!("failed to serialize into JSON");
                    break;
                }
            }
            Some(result) = socket.next() => {
                match result {
                    Ok(Message::Close(_)) => {
                        debug!("received websocket close message");
                        break;
                    }
                    Ok(Message::Text(message)) => {
                        if let Ok(RelayMessage::ProxyResponse { request_id, body, headers, status }) = serde_json::from_slice::<RelayMessage>(message.as_bytes()) {
                            let body_str = std::str::from_utf8(&body).unwrap_or("<binary>");
                            debug!(
                                request_id,
                                status,
                                ?headers,
                                body_str,
                                "received proxy response from client"
                                );
                            if let Some(tx) = state.remove_proxy_request(&request_id).await {
                                let _ = tx.send(RelayMessage::ProxyResponse { request_id, body, headers, status });
                            }
                       } else {
                            error!("failed to deserialize from bytes: {}", message);
                       }
                    }
                    Ok(_) => {},
                    Err(err) => {
                        debug!("received websocket error: {}", err);
                        break;
                    }
                }
            }
        }
    }

    state.remove_client(&client_id).await;

    info!("👨 client disconnected");
}
