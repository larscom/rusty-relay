use axum::{
    Json, Router,
    body::Body,
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing,
};
use axum_server::tls_rustls::RustlsConfig;
use rusty_relay_messages::RelayMessage;
use std::{
    collections::HashMap, fmt::Display, net::SocketAddr, str::FromStr, sync::Arc, time::Duration,
};
use tokio::{
    sync::{Mutex, broadcast, oneshot},
    time,
};
use tokio_stream::StreamExt;

pub struct AppState {
    clients: moka::future::Cache<String, broadcast::Sender<RelayMessage>>,
    proxy_requests: Mutex<HashMap<String, oneshot::Sender<RelayMessage>>>,
    rx_client_evictor: broadcast::Receiver<String>,
    connect_token: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let (tx_client_evictor, rx_client_evictor) = broadcast::channel(100);

        let clients = moka::future::Cache::builder()
            .time_to_live(Duration::from_secs((60 * 60) * 24))
            .eviction_listener(move |client_id: Arc<String>, _, _| {
                let _ = tx_client_evictor.send((*client_id.clone()).to_string());
            })
            .build();

        Self {
            clients,
            proxy_requests: Mutex::new(HashMap::new()),
            rx_client_evictor,
            connect_token: from_env_or_else("CONNECT_TOKEN", || generate_id(24)),
        }
    }

    pub async fn get_client(&self, id: &str) -> Option<broadcast::Sender<RelayMessage>> {
        self.clients.get(id).await
    }

    pub async fn register_client(
        &self,
        id: &str,
    ) -> (
        broadcast::Receiver<RelayMessage>,
        broadcast::Receiver<String>,
    ) {
        let sender = self
            .clients
            .entry(id.to_string())
            .or_insert_with(async { broadcast::channel(100).0 })
            .await
            .into_value();

        (sender.subscribe(), self.rx_client_evictor.resubscribe())
    }
}

async fn get_tls_config() -> Option<RustlsConfig> {
    RustlsConfig::from_pem_file("./certs/cert.pem", "./certs/key.pem")
        .await
        .ok()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("cryptoprovider should be installed");

    let state = Arc::new(AppState::new());

    let router = Router::new()
        .route("/webhook/{client_id}", routing::post(webhook_handler))
        .route("/connect", routing::any(handle_ws_without_id))
        .route("/connect/{client_id}", routing::any(handle_ws_with_id))
        .route(
            "/proxy/{client_id}/{*path}",
            routing::any(proxy_handler_with_path),
        )
        .route(
            "/proxy/{client_id}",
            routing::any(proxy_handler_without_path),
        )
        .route(
            "/proxy/{client_id}/",
            routing::any(proxy_handler_without_path),
        )
        .route("/health", routing::get(health_handler))
        .with_state(state.clone());

    if let Some(tls_config) = get_tls_config().await {
        let addr = SocketAddr::from(([0, 0, 0, 0], from_env_or_else("HTTPS_PORT", || 8443)));
        tracing::info!(
            "🚀 (https) server running on https://{addr}/health - connect token: {}",
            state.connect_token
        );
        axum_server::bind_rustls(addr, tls_config)
            .serve(router.into_make_service())
            .await?;
    } else {
        let addr = SocketAddr::from(([0, 0, 0, 0], from_env_or_else("HTTP_PORT", || 8080)));
        tracing::info!(
            "🚀 (http) server running on http://{addr}/health - connect token: {}",
            state.connect_token
        );
        axum::serve(tokio::net::TcpListener::bind(addr).await?, router).await?
    }

    Ok(())
}

async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "ok").into_response()
}

async fn proxy_handler(
    state: State<Arc<AppState>>,
    client_id: String,
    path: Option<String>,
    headers: HeaderMap,
    method: axum::http::Method,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    let request_id = generate_id(10);
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

async fn proxy_handler_with_path(
    state: State<Arc<AppState>>,
    Path((client_id, path)): Path<(String, String)>,
    headers: HeaderMap,
    method: axum::http::Method,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    proxy_handler(state, client_id, Some(path), headers, method, body).await
}

async fn proxy_handler_without_path(
    state: State<Arc<AppState>>,
    Path(client_id): Path<String>,
    headers: HeaderMap,
    method: axum::http::Method,
    body: axum::body::Bytes,
) -> impl IntoResponse {
    proxy_handler(state, client_id, None, headers, method, body).await
}

async fn webhook_handler(
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

pub async fn handle_ws(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    client_id: String,
    state: Arc<AppState>,
) -> impl IntoResponse {
    match headers.get("PRIVATE-TOKEN") {
        Some(token) => match token.to_str() {
            Ok(token) => {
                if token == state.connect_token {
                    tracing::info!("👨 client connected with id: {client_id}");
                    ws.on_upgrade(move |socket| handle_socket(socket, client_id, state))
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

pub async fn handle_ws_without_id(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let client_id = generate_id(12);
    handle_ws(ws, headers, client_id, state).await
}

pub async fn handle_ws_with_id(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    Path(client_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    handle_ws(ws, headers, client_id, state).await
}

async fn handle_socket(mut socket: WebSocket, client_id: String, state: Arc<AppState>) {
    if let Ok(msg) = serde_json::to_string(&RelayMessage::ClientId(client_id.clone())) {
        if socket.send(Message::Text(msg.into())).await.is_err() {
            tracing::error!("failed to send message to client");
            return;
        }
    } else {
        tracing::error!("failed to serialize into JSON");
        return;
    }

    let (mut rx_relay, mut rx_client_evictor) = state.register_client(&client_id).await;
    let mut ping_interval = time::interval(Duration::from_secs(30));

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
            Ok(rx_client_id) = rx_client_evictor.recv() => {
                if rx_client_id == client_id {
                    tracing::info!("⏰ client id: {client_id} has expired");
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
                       tracing::debug!("received message from client: {}", message);
                       if let Ok(RelayMessage::ProxyResponse { request_id, body, headers, status }) = serde_json::from_slice::<RelayMessage>(message.as_bytes()) {
                        if let Some(tx) = state.proxy_requests.lock().await.remove(&request_id) {
                            let _ = tx.send(RelayMessage::ProxyResponse { request_id, body, headers, status });
                        }
                       } else {
                        tracing::error!("failed to deserialize from bytes");
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

    tracing::info!("👨 client disconnected with id: {client_id}");
}

fn from_env_or_else<T, F>(key: &str, f: F) -> T
where
    T: FromStr + Display,
    F: FnOnce() -> T,
{
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(f)
}

fn generate_id(length: usize) -> String {
    nanoid::nanoid!(length, &nanoid::alphabet::SAFE[2..])
}
