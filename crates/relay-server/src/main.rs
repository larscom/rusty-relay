#![allow(clippy::collapsible_if)]

use axum::{Router, routing};
use std::{net::SocketAddr, sync::Arc};

use crate::{state::AppState, util::from_env_or_else};

mod catch_all;
mod health;
mod proxy;
mod state;
mod tls;
mod util;
mod webhook;
mod websocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tls::init();

    tracing::info!(
        "🦀 Rusty Relay Server :: {} ::",
        from_env_or_else("VERSION", || "0.0.0".to_string())
    );

    let state = Arc::new(AppState::new());

    let router = Router::new()
        .route("/connect", routing::any(websocket::connect_handler))
        .route(
            "/webhook/{client_id}",
            routing::any(webhook::webhook_handler),
        )
        .route(
            "/proxy/{client_id}/{*path}",
            routing::any(proxy::proxy_handler_with_path),
        )
        .route(
            "/proxy/{client_id}",
            routing::any(proxy::proxy_handler_without_path),
        )
        .route(
            "/proxy/{client_id}/",
            routing::any(proxy::proxy_handler_without_path),
        )
        .route("/health", routing::get(health::health_handler))
        .route("/{*path}", routing::any(catch_all::catch_all_handler))
        .with_state(state.clone());

    if let Some(tls_config) = tls::config().await {
        let addr = SocketAddr::from((
            [0, 0, 0, 0],
            from_env_or_else("RUSTY_RELAY_HTTPS_PORT", || 8443),
        ));
        tracing::info!("🚀 server running (https) on https://{addr}/health");
        tracing::info!("🔑 connect token: {}", &state.connect_token);

        axum_server::bind_rustls(addr, tls_config)
            .serve(router.into_make_service())
            .await?;
    } else {
        let addr = SocketAddr::from((
            [0, 0, 0, 0],
            from_env_or_else("RUSTY_RELAY_HTTP_PORT", || 8080),
        ));
        tracing::info!("🚀 server running (http) on http://{addr}/health");
        tracing::info!("🔑 connect token: {}", &state.connect_token);

        axum::serve(tokio::net::TcpListener::bind(addr).await?, router).await?
    }

    Ok(())
}
