use crate::{cli, error, proxy::ProxyHandler, tls, webhook::WebhookHandler};
use futures_util::{SinkExt, StreamExt};
use rusty_relay_messages::RelayMessage;
use tokio_tungstenite::{
    connect_async_tls_with_config,
    tungstenite::{self, Message, Utf8Bytes, client::IntoClientRequest},
};

#[derive(Debug)]
pub struct Client<'a> {
    cli_args: &'a cli::Args,
    proxy_handler: ProxyHandler<'a>,
    webhook_handler: WebhookHandler<'a>,
}

impl<'a> Client<'a> {
    pub fn new(
        args: &'a cli::Args,
        webhook_handler: WebhookHandler<'a>,
        proxy_handler: ProxyHandler<'a>,
    ) -> Self {
        Self {
            cli_args: args,
            webhook_handler,
            proxy_handler,
        }
    }

    pub async fn connect_blocking(&self) -> Result<(), error::Error> {
        let insecure = self.cli_args.insecure;
        let connect = connect_async_tls_with_config;
        let tls_connector = tls::connector(&self.cli_args.ca_cert);

        let ws_proto = if insecure { "ws://" } else { "wss://" };
        let url = format!("{}{}/connect", ws_proto, self.cli_args.server);
        let mut request = url.into_client_request()?;
        request
            .headers_mut()
            .insert("PRIVATE-TOKEN", self.cli_args.token.parse()?);

        match connect(request, None, false, tls_connector).await {
            Ok(ws_stream) => {
                let (mut write, mut read) = ws_stream.0.split();
                while let Some(msg) = read.next().await {
                    if let Ok(Message::Text(message)) = msg {
                        if let Some(response) = self.handle_message(message).await? {
                            write
                                .send(Message::Text(serde_json::to_string(&response)?.into()))
                                .await?;
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

    async fn handle_message(
        &self,
        message: Utf8Bytes,
    ) -> Result<Option<RelayMessage>, error::Error> {
        match serde_json::from_slice::<RelayMessage>(message.as_bytes())? {
            RelayMessage::Webhook {
                method,
                body,
                headers,
            } => {
                self.webhook_handler.handle(method, headers, body).await?;
            }
            RelayMessage::ClientId(ref client_id) => {
                let insecure = self.cli_args.insecure;
                let http_proto = if insecure { "http://" } else { "https://" };

                self.webhook_handler
                    .print_url(client_id, http_proto, &self.cli_args.server);
                self.proxy_handler
                    .print_url(client_id, http_proto, &self.cli_args.server);
            }
            RelayMessage::ProxyRequest {
                request_id,
                path,
                method,
                headers,
                body,
            } => {
                let proxy_response = self
                    .proxy_handler
                    .handle(request_id, path, method, headers, body)
                    .await?;

                return Ok(proxy_response);
            }
            _ => {}
        }

        Ok(None)
    }
}
