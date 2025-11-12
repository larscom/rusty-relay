use thiserror::Error;
use tokio_tungstenite::tungstenite;

#[derive(Error, Debug)]
pub enum Error {
    #[error("into client request error: {0}")]
    InvalidClientRequest(#[from] tungstenite::error::Error),

    #[error("header value error: {0}")]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    #[error("header name error: {0}")]
    InvalidHeaderName(#[from] reqwest::header::InvalidHeaderName),

    #[error("failed to parse relay message: {0}")]
    ParseFailed(#[from] serde_json::Error),

    #[error("http client error: {0}")]
    HttpClientFailed(#[from] reqwest::Error),

    #[error("invalid HTTP method: {0}")]
    InvalidMethod(#[from] tungstenite::http::method::InvalidMethod),
}
