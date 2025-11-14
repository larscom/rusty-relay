use rusty_relay_messages::RelayMessage;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::{Mutex, broadcast, oneshot};

use crate::util::{from_env_or_else, generate_id};

const BROADCAST_SIZE: usize = 100;
const CLIENT_ID_TTL: u64 = (60 * 60) * 24;

pub struct AppState {
    pub clients: moka::future::Cache<String, broadcast::Sender<RelayMessage>>,
    pub proxy_requests: Mutex<HashMap<String, oneshot::Sender<RelayMessage>>>,
    pub rx_client_evictor: broadcast::Receiver<String>,
    pub connect_token: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let (tx_client_evictor, rx_client_evictor) = broadcast::channel(BROADCAST_SIZE);

        let clients = moka::future::Cache::builder()
            .time_to_live(Duration::from_secs(CLIENT_ID_TTL))
            .eviction_listener(move |client_id: Arc<String>, _, _| {
                let _ = tx_client_evictor.send((*client_id.clone()).to_string());
            })
            .build();

        Self {
            clients,
            proxy_requests: Mutex::new(HashMap::new()),
            rx_client_evictor,
            connect_token: from_env_or_else("RUSTY_RELAY_CONNECT_TOKEN", || generate_id(24)),
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
            .or_insert_with(async { broadcast::channel(BROADCAST_SIZE).0 })
            .await
            .into_value();

        (sender.subscribe(), self.rx_client_evictor.resubscribe())
    }
}
