use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub enum HttpError {
    BadRequest(String),
    GatewayTimeout(String),
    Unauthorized(String),
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        match self {
            HttpError::BadRequest(m) => (StatusCode::BAD_REQUEST, m).into_response(),
            HttpError::GatewayTimeout(m) => (StatusCode::GATEWAY_TIMEOUT, m).into_response(),
            HttpError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m).into_response(),
        }
    }
}
