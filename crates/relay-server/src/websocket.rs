use crate::{state::AppState, util::generate_id};
use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use rusty_relay_messages::RelayMessage;
use std::{sync::Arc, time::Duration};
use tokio::time;
use tokio_stream::StreamExt;

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

                    tracing::info!("👨 client connected with id: {client_id}");
                    ws.on_upgrade(move |socket| handle_ws(socket, client_id, state))
                } else {
                    tracing::debug!("❌ client provided invalid token");
                    (StatusCode::UNAUTHORIZED, "Connection token is invalid").into_response()
                }
            }
            Err(_) => {
                tracing::debug!("❌ client provided invalid token format");
                (
                    StatusCode::BAD_REQUEST,
                    "Connection token has invalid format",
                )
                    .into_response()
            }
        },
        None => {
            tracing::debug!("❌ client did not provide token");
            (
                StatusCode::UNAUTHORIZED,
                "Connection token is missing from header",
            )
                .into_response()
        }
    }
}

async fn handle_ws(mut socket: WebSocket, client_id: String, state: State<Arc<AppState>>) {
    if let Ok(msg) = serde_json::to_string(&RelayMessage::ClientId(client_id.clone())) {
        if socket.send(Message::Text(msg.into())).await.is_err() {
            tracing::error!("failed to send message to client");
            return;
        }
    } else {
        tracing::error!("failed to serialize into JSON");
        return;
    }

    let mut rx_relay = state.add_client(&client_id).await;
    let mut ping_interval = time::interval(Duration::from_secs(25));

    loop {
        tokio::select! {
            _ = ping_interval.tick() => {
                if socket.send(Message::Ping(Vec::new().into())).await.is_err() {
                    tracing::error!("failed to send ping to client with id: {client_id}");
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
                        tracing::error!("failed to send message to client");
                        break;
                    }
                } else {
                    tracing::error!("failed to serialize into JSON");
                    break;
                }
            }
            Some(result) = socket.next() => {
                match result {
                    Ok(Message::Close(_)) => {
                        tracing::debug!("received websocket close message");
                        break;
                    }
                    Ok(Message::Text(message)) => {
                        if let Ok(RelayMessage::ProxyResponse { request_id, body, headers, status }) = serde_json::from_slice::<RelayMessage>(message.as_bytes()) {
                            let body_str = std::str::from_utf8(&body).unwrap_or("<binary>");
                            tracing::debug!(
                                "received proxy response from client\nrequest_id: {}\nstatus: {}\nheaders: {:?}\nbody: {}",
                                 request_id,
                                 status,
                                 headers,
                                 body_str
                                );
                            if let Some(tx) = state.remove_proxy_request(&request_id).await {
                                let _ = tx.send(RelayMessage::ProxyResponse { request_id, body, headers, status });
                            }
                       } else {
                            tracing::error!("failed to deserialize from bytes: {}", message);
                       }
                    }
                    Ok(_) => {},
                    Err(err) => {
                        tracing::debug!("received websocket error: {}", err);
                        break;
                    }
                }
            }
        }
    }

    state.remove_client(&client_id).await;

    tracing::info!("👨 client disconnected with id: {client_id}");
}
