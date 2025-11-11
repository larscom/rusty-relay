use futures_util::{SinkExt, StreamExt};
use rusty_relay_messages::RelayMessage;
use tokio_tungstenite::{
    connect_async_tls_with_config,
    tungstenite::{self, Message, client::IntoClientRequest},
};

use crate::{cli, proxy, tls, webhook};

pub async fn connect(args: &cli::Args) -> Result<(), Box<dyn std::error::Error>> {
    let ws_proto = if args.insecure { "ws://" } else { "wss://" };
    let http_proto = if args.insecure { "http://" } else { "https://" };

    let url = format!("{}{}/connect", ws_proto, args.server);
    let mut request = url.into_client_request()?;

    request
        .headers_mut()
        .insert("PRIVATE-TOKEN", args.token.parse()?);

    match connect_async_tls_with_config(request, None, false, tls::connector(&args.ca_cert)).await {
        Ok(ws_stream) => {
            let (mut write, mut read) = ws_stream.0.split();
            while let Some(msg) = read.next().await {
                if let Ok(Message::Text(message)) = msg {
                    match serde_json::from_slice::<RelayMessage>(message.as_bytes())? {
                        RelayMessage::Webhook { ref payload } => {
                            webhook::forward(&args.target, payload).await?;
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
                            let proxy_response = proxy::handle_proxy_request(
                                request_id,
                                path,
                                method,
                                headers,
                                body,
                                args.target.clone(),
                            )
                            .await?;

                            write
                                .send(Message::Text(
                                    serde_json::to_string(&proxy_response)?.into(),
                                ))
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
