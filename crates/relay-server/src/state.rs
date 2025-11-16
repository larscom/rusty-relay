use crate::util::{from_env_or_else, generate_id};
use rusty_relay_messages::RelayMessage;
use std::collections::HashMap;
use tokio::sync::{Mutex, broadcast, oneshot};

pub struct AppState {
    clients: Mutex<HashMap<String, broadcast::Sender<RelayMessage>>>,
    proxy_requests: Mutex<HashMap<String, oneshot::Sender<RelayMessage>>>,
    connect_token: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            clients: Mutex::new(HashMap::new()),
            proxy_requests: Mutex::new(HashMap::new()),
            connect_token: from_env_or_else("RUSTY_RELAY_CONNECT_TOKEN", || generate_id(24)),
        }
    }

    pub async fn add_proxy_request(&self, request_id: &str, tx: oneshot::Sender<RelayMessage>) {
        self.proxy_requests
            .lock()
            .await
            .insert(request_id.to_string(), tx);
    }

    pub async fn remove_proxy_request(
        &self,
        request_id: &str,
    ) -> Option<oneshot::Sender<RelayMessage>> {
        self.proxy_requests.lock().await.remove(request_id)
    }

    pub fn connect_token(&self) -> &str {
        self.connect_token.as_str()
    }

    pub async fn remove_client(&self, id: &str) {
        self.clients.lock().await.remove(id);
    }

    pub async fn get_client(&self, id: &str) -> Option<broadcast::Sender<RelayMessage>> {
        self.clients.lock().await.get(id).cloned()
    }

    pub async fn add_client(&self, id: &str) -> broadcast::Receiver<RelayMessage> {
        let mut clients = self.clients.lock().await;
        let sender = clients
            .entry(id.to_string())
            .or_insert_with(|| broadcast::channel(100).0);

        sender.subscribe()
    }
}
