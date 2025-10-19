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
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use tokio_stream::StreamExt;

#[derive(Default)]
pub struct AppState {
    // TODO: entries need to be removed after 48hrs
    clients: Mutex<HashMap<String, broadcast::Sender<serde_json::Value>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_sender(&self, id: &str) -> Option<broadcast::Sender<serde_json::Value>> {
        self.clients.lock().unwrap().get(id).cloned()
    }

    pub fn register(&self, id: &str) -> broadcast::Receiver<serde_json::Value> {
        let mut clients = self.clients.lock().unwrap();
        let sender = clients
            .entry(id.to_string())
            .or_insert_with(|| broadcast::channel(100).0);
        sender.subscribe()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let state = Arc::new(AppState::new());
    let app = Router::new()
        .route("/webhook/{id}", routing::post(receive_webhook))
        .route("/ws/{id}", routing::any(handle_ws))
        .with_state(state.clone());

    let addr = "0.0.0.0:8080";
    tracing::info!("🚀 relay server running on {addr}");
    Ok(axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?)
}

async fn receive_webhook(
    Path(id): Path<String>,
    state: State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    tracing::info!("📩 webhook received for {id} with payload: {payload}");

    if let Some(sender) = state.get_sender(&id) {
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
    let mut rx = state.register(&id);

    loop {
        tokio::select! {
            Ok(payload) = rx.recv() => {
                if socket
                    .send(Message::Text(payload.to_string().into()))
                    .await
                    .is_err()
                {
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
