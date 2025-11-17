use axum::{Json, response::IntoResponse};
use serde_json::json;

pub async fn health_handler() -> impl IntoResponse {
    Json(json!({
        "status": "UP"
    }))
}
