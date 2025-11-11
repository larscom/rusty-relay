use std::{collections::HashMap, fs, str::FromStr, sync::Arc};

use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use reqwest::{
    Client, Method,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use rustls::pki_types::pem::PemObject;
use rusty_relay_messages::RelayMessage;
use tokio_tungstenite::{
    Connector, connect_async_tls_with_config,
    tungstenite::{self, Message, client::IntoClientRequest},
};

#[derive(Parser, Debug)]
#[command(name = "rusty-relay")]
struct Args {
    #[arg(long)]
    /// The rusty-relay-server hostname e.g: localhost:8080 or my.server.com
    server: String,

    #[arg(long)]
    /// The connection token generated on rusty-relay-server
    token: String,

    #[arg(long)]
    /// Target URL to the local webserver e.g: http://localhost:3000/api/hook
    target: String,

    #[arg(long)]
    /// Connect to the server without TLS
    insecure: bool,

    #[arg(long)]
    /// Path to CA certificate (PEM encoded)
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
            Err(_) => {
                println!("⚠️ WARNING: ca certificate was not found at: {cert_path}");
                None
            }
        })
}

async fn connect(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let ws_proto = if args.insecure { "ws://" } else { "wss://" };
    let http_proto = if args.insecure { "http://" } else { "https://" };

    let url = format!("{}{}/connect", ws_proto, args.server);
    let mut request = url.into_client_request()?;

    request
        .headers_mut()
        .insert("PRIVATE-TOKEN", args.token.parse()?);

    match connect_async_tls_with_config(request, None, false, get_tls_connector(&args.ca_cert))
        .await
    {
        Ok(ws_stream) => {
            let (mut write, mut read) = ws_stream.0.split();
            while let Some(msg) = read.next().await {
                if let Ok(Message::Text(message)) = msg {
                    match serde_json::from_slice::<RelayMessage>(message.as_bytes())? {
                        RelayMessage::Webhook { ref payload } => {
                            forward(&args.target, payload).await?;
                        }
                        RelayMessage::ClientId(client_id) => {
                            let webhook_url =
                                format!("{}{}/webhook/{}", http_proto, args.server, client_id);
                            let proxy_url =
                                format!("{}{}/proxy/{}", http_proto, args.server, client_id);
                            println!("✅ You can send webhook requests to: {webhook_url}");
                            println!("✅ You can send proxy requests to: {proxy_url}")
                        }
                        RelayMessage::ProxyRequest {
                            request_id,
                            path,
                            method,
                            headers,
                            body,
                        } => {
                            let client = Client::builder().use_rustls_tls().build()?;
                            let url = if let Some(p) = path.as_ref() {
                                format!("{}/{}", &args.target, p)
                            } else {
                                args.target.clone()
                            };

                            let mut request_headers = HeaderMap::with_capacity(headers.len());
                            for (k, v) in headers {
                                request_headers
                                    .insert(k.parse::<HeaderName>()?, v.parse::<HeaderValue>()?);
                            }

                            let res = client
                                .request(Method::from_str(&method)?, url)
                                .headers(request_headers)
                                .body(body)
                                .send()
                                .await?;

                            let mut response_headers = HashMap::new();
                            for (k, v) in res.headers() {
                                if let Ok(value) = v.to_str() {
                                    response_headers.insert(k.to_string(), value.to_string());
                                }
                            }
                            let status = res.status().as_u16();

                            let message = RelayMessage::ProxyResponse {
                                request_id,
                                body: res.bytes().await?.to_vec(),
                                headers: response_headers,
                                status,
                            };
                            write
                                .send(Message::Text(serde_json::to_string(&message)?.into()))
                                .await
                                .expect("failed to send message");
                        }
                        _ => {}
                    }
                }
            }
        }
        Err(tungstenite::Error::Http(response)) => {
            if let Some(body) = response.body() {
                println!("❌ ERROR: {}", String::from_utf8_lossy(body));
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
