use std::{collections::HashMap, str::FromStr};

use anyhow::Context;
use reqwest::{
    Client, Method,
    header::{HeaderMap, HeaderName, HeaderValue},
};

#[derive(Debug)]
pub struct WebhookHandler<'a> {
    target: &'a str,
    http_client: Client,
}

impl<'a> WebhookHandler<'a> {
    pub fn new(target: &'a str, http_client: Client) -> Self {
        Self {
            target,
            http_client,
        }
    }

    pub async fn handle(
        &self,
        method: String,
        headers: HashMap<String, String>,
        body: Vec<u8>,
    ) -> anyhow::Result<()> {
        let mut request_headers = HeaderMap::with_capacity(headers.len());
        for (k, v) in headers {
            request_headers.insert(
                k.parse::<HeaderName>().with_context(|| {
                    format!("failed to parse key: {} as header name for webhook", &k)
                })?,
                v.parse::<HeaderValue>().with_context(|| {
                    format!("failed to parse value: {} as header value for webhook", &v)
                })?,
            );
        }

        let response = self
            .http_client
            .request(
                Method::from_str(&method)
                    .with_context(|| format!("failed to parse http method: {}", &method))?,
                self.target,
            )
            .headers(request_headers)
            .body(body)
            .send()
            .await
            .map_err(|err| {
                println!(
                    "⚠️ WARNING: request ({method}) to {} failed: {err}",
                    self.target
                )
            });

        if let Ok(res) = response {
            println!(
                "➡️ forwarded webhook ({}) to {}, got {}",
                method,
                self.target,
                res.status()
            );

            if res.status().is_client_error() || res.status().is_server_error() {
                println!(
                    "❌ ERROR:\n{}",
                    res.text()
                        .await
                        .context("failed to parse webhook response body text")?
                );
            }
        }

        Ok(())
    }

    pub fn print_url(&self, client_id: &str, protocol: &str, server: &str) {
        let webhook_url = format!("{}{}/webhook/{}", protocol, server, client_id);
        println!("✅ You can send webhooks to: {webhook_url}");
    }
}
