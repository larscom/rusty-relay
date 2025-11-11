mod cli;
mod error;
mod proxy;
mod tls;
mod webhook;
mod websocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tls::init();

    let args = cli::args();
    let ws_client = websocket::Client::from_args(&args);

    ws_client.connect_blocking().await?;

    Ok(())
}
