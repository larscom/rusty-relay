use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelayMessage {
    Webhook {
        payload: String,
    },
    ClientId(String),
    ProxyRequest {
        request_id: String,
        method: String,
        headers: HashMap<String, String>,
        body: Vec<u8>,
        path: Option<String>,
    },
    ProxyResponse {
        request_id: String,
        headers: HashMap<String, String>,
        body: Vec<u8>,
        status: u16,
    },
}
