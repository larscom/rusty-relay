#![allow(clippy::collapsible_if)]

use crate::{proxy::ProxyHandler, webhook::WebhookHandler};

mod cli;
mod error;
mod proxy;
mod tls;
mod version;
mod webhook;
mod websocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tls::init();

    if version::print_version() {
        return Ok(());
    }

    let args = cli::args();

    let http_client = reqwest::Client::builder().use_rustls_tls().build()?;
    let webhook_handler = WebhookHandler::new(&args.target, http_client.clone());
    let proxy_handler = ProxyHandler::new(&args.target, http_client);

    let ws_client = websocket::Client::new(&args, webhook_handler, proxy_handler);

    ws_client.connect_blocking().await?;

    Ok(())
}
