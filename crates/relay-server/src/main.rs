use axum::{
    Json, Router,
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing,
};
use axum_server::tls_rustls::RustlsConfig;
use rusty_relay_shared::RelayMessage;
use std::{fmt::Display, net::SocketAddr, str::FromStr, sync::Arc, time::Duration};
use tokio::{sync::broadcast, time::interval};
use tokio_stream::StreamExt;

pub struct AppState {
    clients: moka::future::Cache<String, broadcast::Sender<serde_json::Value>>,
    client_evictor: broadcast::Receiver<String>,
    token: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let (tx, client_evictor) = broadcast::channel(100);

        let clients = moka::future::Cache::builder()
            .time_to_live(Duration::from_secs((60 * 60) * 24))
            .eviction_listener(move |key: Arc<String>, _, _| {
                let _ = tx.send((*key.clone()).to_string());
            })
            .build();

        Self {
            clients,
            client_evictor,
            token: from_env_or_default("CONNECT_TOKEN", || {
                nanoid::nanoid!(24, &nanoid::alphabet::SAFE[2..])
            }),
        }
    }

    pub async fn get_sender(&self, id: &str) -> Option<broadcast::Sender<serde_json::Value>> {
        self.clients.get(id).await
    }

    pub async fn register(
        &self,
        id: &str,
    ) -> (
        broadcast::Receiver<serde_json::Value>,
        broadcast::Receiver<String>,
    ) {
        let sender = self
            .clients
            .entry(id.to_string())
            .or_insert_with(async { broadcast::channel(100).0 })
            .await
            .into_value();

        (sender.subscribe(), self.client_evictor.resubscribe())
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
        .route("/webhook/{id}", routing::post(webhook_handler))
        .route("/connect", routing::any(handle_ws_without_id))
        .route("/connect/{id}", routing::any(handle_ws_with_id))
        .route("/health", routing::get(health_handler))
        .with_state(state.clone());

    if let Some(tls_config) = get_tls_config().await {
        let addr = SocketAddr::from(([0, 0, 0, 0], from_env_or_default("HTTPS_PORT", || 8443)));
        tracing::info!(
            "🚀 server running on https://{addr} - connect token: {}",
            state.token
        );
        axum_server::bind_rustls(addr, tls_config)
            .serve(router.into_make_service())
            .await?;
    } else {
        let addr = SocketAddr::from(([0, 0, 0, 0], from_env_or_default("HTTP_PORT", || 8080)));
        tracing::info!(
            "🚀 server running on http://{addr} - connect token: {}",
            state.token
        );
        axum::serve(tokio::net::TcpListener::bind(addr).await?, router).await?
    }

    Ok(())
}

async fn health_handler() -> impl IntoResponse {
    StatusCode::OK
}

async fn webhook_handler(
    Path(id): Path<String>,
    state: State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    tracing::info!("📩 webhook received for {id}");
    tracing::debug!("{}", payload);

    if let Some(sender) = state.get_sender(&id).await {
        let _ = sender.send(payload);
    }

    StatusCode::OK
}

pub async fn handle_ws(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    id: String,
    state: Arc<AppState>,
) -> impl IntoResponse {
    match headers.get("PRIVATE-TOKEN") {
        Some(token) => match token.to_str() {
            Ok(token) => {
                if token == state.token {
                    tracing::info!("🔗 client connected to {id}");
                    ws.on_upgrade(move |socket| handle_socket(socket, id, state))
                } else {
                    tracing::debug!("❌ client provided invalid token value");
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
    let id = nanoid::nanoid!(12, &nanoid::alphabet::SAFE[2..]);
    handle_ws(ws, headers, id, state).await
}

pub async fn handle_ws_with_id(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    handle_ws(ws, headers, id, state).await
}

async fn handle_socket(mut socket: WebSocket, id: String, state: Arc<AppState>) {
    if let Ok(msg) = serde_json::to_string(&RelayMessage::ClientId(id.clone())) {
        if socket.send(Message::Text(msg.into())).await.is_err() {
            tracing::error!("failed to send RelayMessage");
            return;
        }
    } else {
        tracing::error!("failed to serialize RelayMessage into JSON");
        return;
    }

    let (mut rx_payload, mut rx_client) = state.register(&id).await;
    let mut ping_interval = interval(Duration::from_secs(30));

    loop {
        tokio::select! {
            _ = ping_interval.tick() => {
                if socket.send(Message::Ping(Vec::new().into())).await.is_err() {
                    tracing::error!("failed to send ping");
                    break;
                }
            }
            Ok(payload) = rx_payload.recv() => {
                if let Ok(msg) = serde_json::to_string(&RelayMessage::Forward(payload.to_string())) {
                    if socket
                        .send(Message::Text(msg.into()))
                        .await
                        .is_err()
                    {
                        tracing::error!("failed to send RelayMessage");
                        break;
                    }
                } else {
                    tracing::error!("failed to serialize RelayMessage into JSON");
                    break;
                }
            }
            Ok(webhook_id) = rx_client.recv() => {
                if webhook_id == id {
                    tracing::info!("⏰ webhook {id} has expired");
                    break;
                }
            }
            Some(result) = socket.next() => {
                if result.is_err() {
                    tracing::debug!("received websocket error: {}", result.unwrap_err());
                    break;
                }
                if let Ok(msg) = result && let Message::Close(_) = msg {
                    tracing::debug!("received websocket close message");
                    break;
                }
            }
        }
    }

    tracing::info!("🔗 client disconnected from {id}");
}

fn from_env_or_default<T>(key: &str, default: fn() -> T) -> T
where
    T: FromStr + Display,
{
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(default)
}
