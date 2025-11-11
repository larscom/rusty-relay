use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use rusty_relay_messages::RelayMessage;
use tokio::sync::oneshot;

use crate::{state::AppState, util::generate_id};

pub async fn proxy_handler_with_path(
    state: State<Arc<AppState>>,
    Path((client_id, path)): Path<(String, String)>,
    headers: HeaderMap,
    method: axum::http::Method,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    proxy_handler(state, client_id, Some(path), headers, method, body).await
}

pub async fn proxy_handler_without_path(
    state: State<Arc<AppState>>,
    Path(client_id): Path<String>,
    headers: HeaderMap,
    method: axum::http::Method,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    proxy_handler(state, client_id, None, headers, method, body).await
}

pub async fn proxy_handler(
    state: State<Arc<AppState>>,
    client_id: String,
    path: Option<String>,
    headers: HeaderMap,
    method: axum::http::Method,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let request_id = generate_id(20);
    tracing::info!("🖥 proxy request ({request_id}) received for client id: {client_id}");

    if let Some(sender) = state.get_client(&client_id).await {
        let _ = sender.send(RelayMessage::ProxyRequest {
            request_id: request_id.clone(),
            path,
            method: method.to_string(),
            headers: headers
                .iter()
                .filter_map(|(k, v)| {
                    v.to_str()
                        .ok()
                        .map(|v| v.to_string())
                        .map(|v| (k.to_string(), v))
                })
                .collect(),
            body: body.to_vec(),
        });
    }

    let (resp_tx, resp_rx) = oneshot::channel();

    {
        state
            .proxy_requests
            .lock()
            .await
            .insert(request_id, resp_tx);
    }

    match tokio::time::timeout(std::time::Duration::from_secs(5), resp_rx).await {
        Ok(Ok(RelayMessage::ProxyResponse {
            body,
            headers,
            status,
            ..
        })) => {
            let mut response = axum::response::Response::builder().status(status);
            for (k, v) in headers.iter().filter(|(k, _)| *k != "content-length") {
                response = response.header(k, v);
            }

            let content_type = headers.get("content-type");
            match content_type {
                Some(ct) => {
                    if ct.contains("text/html") {
                        let html = regex::Regex::new(r#"(src|href)="(/?)([^"]*)""#)
                            .expect("valid regex")
                            .replace_all(
                                &String::from_utf8_lossy(&body),
                                format!(r#"$1="/proxy/{client_id}/$3""#),
                            )
                            .into_owned();

                        response
                            .body(Body::from(html))
                            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
                            .into_response()
                    } else {
                        response
                            .body(Body::from(body))
                            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
                            .into_response()
                    }
                }
                None => response
                    .body(Body::from(body))
                    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
                    .into_response(),
            }
        }
        _ => (StatusCode::GATEWAY_TIMEOUT, "Timeout").into_response(),
    }
}
