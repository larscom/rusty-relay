use thiserror::Error;
use tokio_tungstenite::tungstenite;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Into client request error: {0}")]
    IntoClientRequest(#[from] tungstenite::error::Error),

    #[error("HeaderValue error: {0}")]
    HeaderValue(#[from] reqwest::header::InvalidHeaderValue),
}
