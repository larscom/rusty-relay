use crate::{
    error::HttpError,
    state::AppState,
    util::{self, generate_id},
};
use axum::{
    body::{Body, Bytes},
    extract::{Path, Query, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::CookieJar;
use rusty_relay_messages::RelayMessage;
use std::{ops::Not, sync::Arc};
use tokio::sync::oneshot;
use tracing::{debug, info};

#[tracing::instrument(skip(state, jar, body))]
pub async fn catch_all_handler(
    state: State<Arc<AppState>>,
    headers: HeaderMap,
    method: Method,
    Path(path): Path<String>,
    Query(params): Query<Vec<(String, String)>>,
    jar: CookieJar,
    body: Bytes,
) -> impl IntoResponse {
    let request_id = generate_id(20);
    info!(request_id, "🖥 proxy request received");

    if let Some(client_id) = jar.get("client_id") {
        if let Some(sender) = state.get_client(client_id.value()).await {
            let query = params.is_empty().not().then(|| util::get_query(params));
            let _ = sender.send(RelayMessage::ProxyRequest {
                request_id: request_id.clone(),
                path: Some(path),
                query,
                method: method.to_string(),
                headers: util::into_hashmap(headers),
                body: body.to_vec(),
            });
        }

        let (resp_tx, resp_rx) = oneshot::channel();

        state.add_proxy_request(&request_id, resp_tx).await;

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

                response
                    .body(Body::from(body))
                    .map_err(|e| HttpError::BadRequest(e.to_string()))
                    .into_response()
            }
            _ => HttpError::GatewayTimeout("Timeout".to_string()).into_response(),
        }
    } else {
        StatusCode::OK.into_response()
    }
}

#[tracing::instrument(skip(jar))]
pub async fn root_handler(
    Query(params): Query<Vec<(String, String)>>,
    jar: CookieJar,
) -> impl IntoResponse {
    if let Some(client_id) = jar.get("client_id") {
        let client_id = client_id.value();
        let path = format!("/proxy/{client_id}");

        let qs = util::get_query(params);
        let url = format!("{path}{qs}");

        debug!("client_id: {client_id} incoming path: / redirecting to: {url}");
        Redirect::temporary(url.as_str()).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}
