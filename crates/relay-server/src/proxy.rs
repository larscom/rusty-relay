use crate::{
    error::HttpError,
    state::AppState,
    util::{self, generate_id},
};
use axum::{
    body::Body,
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, Expiration},
};
use rusty_relay_messages::RelayMessage;
use std::sync::Arc;
use tokio::sync::oneshot;
use tracing::info;

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

#[tracing::instrument(skip(state))]
pub async fn proxy_handler(
    state: State<Arc<AppState>>,
    client_id: String,
    path: Option<String>,
    headers: HeaderMap,
    method: axum::http::Method,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let request_id = generate_id(20);
    info!(request_id, "🖥 proxy request received");

    if let Some(sender) = state.get_client(&client_id).await {
        let _ = sender.send(RelayMessage::ProxyRequest {
            request_id: request_id.clone(),
            path,
            method: method.to_string(),
            headers: util::into_hashmap(headers),
            body: body.to_vec(),
        });
    } else {
        return ProxyResponse::new(
            CookieJar::default(),
            HttpError::BadRequest(format!("Client id is unknown: {}", client_id)),
        );
    }

    let (resp_tx, resp_rx) = oneshot::channel();

    state.add_proxy_request(&request_id, resp_tx).await;

    let client_id_cookie = Cookie::build(("client_id", client_id.clone()))
        .expires(Expiration::Session)
        .path("/")
        .http_only(true)
        .build();

    let cookie_jar = CookieJar::new().add(client_id_cookie);

    match tokio::time::timeout(state.proxy_timeout(), resp_rx).await {
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
            ProxyResponse::new(
                cookie_jar,
                response
                    .body(Body::from(body))
                    .map_err(|e| HttpError::BadRequest(e.to_string())),
            )
        }
        _ => ProxyResponse::new(cookie_jar, HttpError::GatewayTimeout("Timeout".to_string())),
    }
}

pub struct ProxyResponse {
    cookie_jar: CookieJar,
    response: Response,
}

impl ProxyResponse {
    pub fn new(cookie_jar: CookieJar, response: impl IntoResponse) -> Self {
        Self {
            cookie_jar,
            response: response.into_response(),
        }
    }
}

impl IntoResponse for ProxyResponse {
    fn into_response(self) -> Response {
        (self.cookie_jar, self.response).into_response()
    }
}
