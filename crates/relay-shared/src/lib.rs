use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum RelayMessage {
    Forward(String),
    ClientId(String),
}
