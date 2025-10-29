use axum::{
    Json, Router,
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
    response::IntoResponse,
    routing,
};
use axum_server::tls_rustls::RustlsConfig;
use std::{fmt::Display, net::SocketAddr, str::FromStr, sync::Arc, time::Duration};
use tokio::sync::broadcast;
use tokio_stream::StreamExt;

pub struct AppState {
    clients: moka::future::Cache<String, broadcast::Sender<serde_json::Value>>,
    client_evictor: broadcast::Receiver<String>,
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
        .route("/webhook/{id}", routing::post(receive_webhook))
        .route("/connect/{id}", routing::any(handle_ws))
        .with_state(state.clone());

    if let Some(tls_config) = get_tls_config().await {
        let addr = SocketAddr::from(([0, 0, 0, 0], from_env_or_default("HTTPS_PORT", 8443)));
        tracing::info!("🚀 relay (https) server running on {addr}");
        axum_server::bind_rustls(addr, tls_config)
            .serve(router.into_make_service())
            .await?;
    } else {
        let addr = SocketAddr::from(([0, 0, 0, 0], from_env_or_default("HTTP_PORT", 8080)));
        tracing::info!("🚀 relay (http) server running on {addr}");
        axum::serve(tokio::net::TcpListener::bind(addr).await?, router).await?
    }

    Ok(())
}

async fn receive_webhook(
    Path(id): Path<String>,
    state: State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    tracing::info!("📩 webhook received for {id} with payload: {payload}");

    if let Some(sender) = state.get_sender(&id).await {
        let _ = sender.send(payload);
    }

    StatusCode::OK
}

pub async fn handle_ws(
    ws: WebSocketUpgrade,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    tracing::info!("🔗 client connected to {id}");

    ws.on_upgrade(move |socket| handle_socket(socket, id, state))
}

async fn handle_socket(mut socket: WebSocket, id: String, state: Arc<AppState>) {
    let (mut rx_payload, mut rx_client) = state.register(&id).await;

    loop {
        tokio::select! {
            Ok(payload) = rx_payload.recv() => {
                if socket
                    .send(Message::Text(payload.to_string().into()))
                    .await
                    .is_err()
                {
                    break;
                }
            },
            Ok(webhook_id) = rx_client.recv() => {
                if webhook_id == id {
                    tracing::info!("⏰ webhook {id} has expired");
                    break;
                }
            },
            Some(result) = socket.next() => {
                if result.is_err() {
                    break;
                }
                if let Ok(msg) = result && let Message::Close(_) = msg {
                    break;
                }
            }
        }
    }

    tracing::info!("🔗 client disconnected from {id}");
}

fn from_env_or_default<T>(key: &str, default: T) -> T
where
    T: FromStr + Display,
{
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}
