use std::{fs, sync::Arc};

use clap::Parser;
use futures_util::StreamExt;
use reqwest::Client;
use rustls::pki_types::pem::PemObject;
use tokio_tungstenite::{Connector, connect_async_tls_with_config, tungstenite::Message};

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
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("cryptoprovider should be installed");

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

fn get_tls_connector() -> Option<Connector> {
    match fs::read("ca.pem") {
        Ok(ca_cert) => {
            let mut root_store = rustls::RootCertStore::empty();
            let pem = rustls::pki_types::CertificateDer::from_pem_slice(&ca_cert)
                .expect("cert should be valid pem");

            root_store.add(pem).expect("cert should be added");

            let config = rustls::ClientConfig::builder()
                .with_root_certificates(root_store)
                .with_no_client_auth();

            Some(Connector::Rustls(Arc::new(config)))
        }
        Err(_) => None,
    }
}

async fn connect(
    server: String,
    id: String,
    target: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("{server}/ws/{id}");
    let (mut ws_stream, _) =
        connect_async_tls_with_config(url, None, false, get_tls_connector()).await?;

    while let Some(msg) = ws_stream.next().await {
        if let Ok(Message::Text(payload)) = msg {
            forward(&target, &payload).await?;
        }
    }

    Ok(())
}

async fn forward(target: &str, payload: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder().use_rustls_tls().build()?;
    let res = client.post(target).body(payload.to_string()).send().await?;
    tracing::info!(
        "➡️  forwarded webhook, got {}\n{}",
        res.status(),
        res.text().await?
    );
    Ok(())
}
