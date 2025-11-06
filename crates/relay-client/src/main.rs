use std::{fs, sync::Arc};

use clap::Parser;
use futures_util::StreamExt;
use reqwest::Client;
use rustls::pki_types::pem::PemObject;
use rusty_relay_shared::RelayMessage;
use tokio_tungstenite::{
    Connector, connect_async_tls_with_config,
    tungstenite::{self, Message, client::IntoClientRequest},
};

#[derive(Parser, Debug)]
#[command(name = "rusty-relay")]
struct Args {
    #[arg(long)]
    /// The server hostname e.g: localhost:8080 or my.server.com
    hostname: String,

    #[arg(long)]
    /// The connection token generated on the server
    token: String,

    #[arg(long)]
    /// Unique ID to which a client can connect and webhooks gets send to. Multiple clients can connect to the same ID.
    id: Option<String>,

    #[arg(long)]
    /// Target URL to the local webserver e.g: http://localhost:3000/api/hook
    target: String,

    #[arg(long)]
    /// Connect to the server without TLS (default: false)
    insecure: bool,

    #[arg(long)]
    /// Full path to custom CA certificate
    ca_cert: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("cryptoprovider should be installed");

    connect(&Args::parse()).await?;

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
    let ws_proto = if args.insecure { "ws://" } else { "wss://" };
    let http_proto = if args.insecure { "http://" } else { "https://" };

    let url = if let Some(id) = args.id.as_ref() {
        format!("{}{}/connect/{}", ws_proto, args.hostname, id)
    } else {
        format!("{}{}/connect", ws_proto, args.hostname)
    };

    let mut request = url.into_client_request()?;

    request
        .headers_mut()
        .insert("PRIVATE-TOKEN", args.token.parse()?);

    match connect_async_tls_with_config(request, None, false, get_tls_connector(&args.ca_cert))
        .await
    {
        Ok(ws_stream) => {
            let (mut ws_stream, _) = ws_stream;
            while let Some(msg) = ws_stream.next().await {
                if let Ok(Message::Text(message)) = msg {
                    match serde_json::from_slice::<RelayMessage>(message.as_bytes())? {
                        RelayMessage::Webhook { ref payload } => {
                            forward(&args.target, payload).await?;
                        }
                        RelayMessage::ClientId(client_id) => {
                            let url =
                                format!("{}{}/webhook/{}", http_proto, args.hostname, client_id);
                            println!("✅ You can send webhooks to this url: {url}")
                        }
                    }
                }
            }
        }
        Err(tungstenite::Error::Http(response)) => {
            if let Some(body) = response.body() {
                println!("ERROR: {}", String::from_utf8_lossy(body));
            }
        }
        Err(err) => return Err(err.into()),
    }

    Ok(())
}

async fn forward(target: &str, payload: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder().use_rustls_tls().build()?;
    let res = client.post(target).body(payload.to_string()).send().await?;
    println!("➡️ forwarded webhook to {}, got {}", target, res.status());
    if res.status().is_client_error() || res.status().is_server_error() {
        println!("{}", res.text().await?)
    }
    Ok(())
}
