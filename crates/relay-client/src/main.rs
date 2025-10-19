use clap::Parser;
use futures_util::StreamExt;
use reqwest::Client;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Parser, Debug)]
#[command(name = "rusty-relay")]
struct Args {
    #[arg(long)]
    server: String,
    #[arg(long)]
    id: String,
    #[arg(long)]
    target: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    tracing::info!(
        "🔗 connecting to relay server {} with id {} forwarding to: {}",
        args.server,
        args.id,
        args.target
    );
    connect(args.server, args.id, args.target).await?;
    Ok(())
}

pub async fn connect(
    server: String,
    id: String,
    target: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{server}/ws/{id}");
    let (mut ws_stream, _) = connect_async(url).await?;

    while let Some(msg) = ws_stream.next().await {
        if let Ok(Message::Text(payload)) = msg {
            forward(&target, &payload).await?;
        }
    }

    Ok(())
}

pub async fn forward(target: &str, payload: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder().use_rustls_tls().build()?;
    let res = client.post(target).body(payload.to_string()).send().await?;
    tracing::info!(
        "➡️  forwarded webhook, got {}\n{}",
        res.status(),
        res.text().await?
    );
    Ok(())
}
