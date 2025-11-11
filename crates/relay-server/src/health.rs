use axum::{http::StatusCode, response::IntoResponse};

pub async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "ok").into_response()
}
