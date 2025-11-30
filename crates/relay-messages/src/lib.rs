use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelayMessage {
    Webhook {
        method: String,
        headers: HashMap<String, String>,
        body: Vec<u8>,
    },
    ClientId(String),
    ProxyRequest {
        request_id: String,
        method: String,
        headers: HashMap<String, String>,
        body: Vec<u8>,
        path: Option<String>,
        query: Option<String>,
    },
    ProxyResponse {
        request_id: String,
        headers: HashMap<String, String>,
        body: Vec<u8>,
        status: u16,
    },
}
