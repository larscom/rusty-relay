use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelayMessage {
    Webhook {
        payload: String,
    },
    ClientId(String),
    ProxyRequest {
        request_id: String,
        path: Option<String>,
    },
    ProxyResponse {
        request_id: String,
        body: String,
    },
}
