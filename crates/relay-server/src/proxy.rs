use crate::{
    state::AppState,
    util::{self, generate_id},
};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, Expiration},
};
use rusty_relay_messages::RelayMessage;
use std::sync::Arc;
use tokio::sync::oneshot;

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
) -> (CookieJar, Response) {
    let request_id = generate_id(20);
    tracing::info!("🖥 proxy request ({request_id}) received for client id: {client_id}");

    if let Some(sender) = state.get_client(&client_id).await {
        let _ = sender.send(RelayMessage::ProxyRequest {
            request_id: request_id.clone(),
            path,
            method: method.to_string(),
            headers: util::into_hashmap(headers),
            body: body.to_vec(),
        });
    } else {
        return (
            CookieJar::default(),
            (
                StatusCode::BAD_REQUEST,
                format!("Client id is unknown: {}", client_id),
            )
                .into_response(),
        );
    }

    let (resp_tx, resp_rx) = oneshot::channel();

    {
        state
            .proxy_requests
            .lock()
            .await
            .insert(request_id, resp_tx);
    }

    let client_id_cookie = Cookie::build(("client_id", client_id.clone()))
        .expires(Expiration::Session)
        .path("/")
        .http_only(true)
        .build();

    let cookie_jar = CookieJar::new().add(client_id_cookie);

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
            (
                cookie_jar,
                response
                    .body(Body::from(body))
                    .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
                    .into_response(),
            )
        }
        _ => (
            cookie_jar,
            (StatusCode::GATEWAY_TIMEOUT, "Timeout").into_response(),
        ),
    }
}
