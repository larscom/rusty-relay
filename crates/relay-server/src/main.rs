use axum::{Router, response::IntoResponse, routing};

async fn handler() -> impl IntoResponse {
    "Hello!"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // let state = Arc::new(AppState::new());
    let app = Router::new().route("/hello", routing::get(handler));
    // .route("/webhook/:id", post(receive_webhook))
    // .route("/connect/:id", axum::routing::get(ws::handle_ws))
    // .with_state(state.clone());

    let addr = "0.0.0.0:8080";
    tracing::info!("🚀 relay server running on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    Ok(axum::serve(listener, app).await?)
}
