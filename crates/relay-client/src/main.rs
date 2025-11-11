mod cli;
mod proxy;
mod tls;
mod webhook;
mod websocket;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tls::init();

    websocket::connect(&cli::args()).await?;

    Ok(())
}
