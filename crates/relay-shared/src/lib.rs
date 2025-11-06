use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum RelayMessage {
    Webhook { payload: String },
    ClientId(String),
}
