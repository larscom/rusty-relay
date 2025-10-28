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
    /// The server hostname e.g: localhost:8080 or my.server.com
    hostname: String,

    #[arg(long)]
    /// Connect to the server without TLS (default: false)
    insecure: bool,

    #[arg(long)]
    /// Unique ID to which a client can connect and webhooks gets send to
    id: String,

    #[arg(long)]
    /// Target URL to the local webserver e.g: http://localhost:3000/api/hook
    target: String,

    #[arg(long)]
    /// Path to custom CA certificate
    ca_cert: Option<String>,
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
        args.hostname,
        args.id,
        args.target
    );
    connect(&args).await?;
    Ok(())
}

fn get_tls_connector(ca_cert_path: &Option<String>) -> Option<Connector> {
    ca_cert_path
        .as_ref()
        .and_then(|cert_path| match fs::read(cert_path) {
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
        })
}

async fn connect(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let protocol = if args.insecure { "ws://" } else { "wss://" };

    let url = format!("{}{}/ws/{}", protocol, args.hostname, args.id);
    let (mut ws_stream, _) =
        connect_async_tls_with_config(url, None, false, get_tls_connector(&args.ca_cert)).await?;

    while let Some(msg) = ws_stream.next().await {
        if let Ok(Message::Text(payload)) = msg {
            forward(&args.target, &payload).await?;
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
