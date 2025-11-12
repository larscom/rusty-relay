use reqwest::Client;

use crate::error;

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

    pub async fn handle(&self, payload: &str) -> Result<(), error::Error> {
        let response: Result<reqwest::Response, ()> = self
            .http_client
            .post(self.target)
            .body(payload.to_string())
            .send()
            .await
            .map_err(|err| println!("⚠️ WARNING: request to {} failed: {err}", &self.target));

        if let Ok(res) = response {
            println!(
                "➡️ forwarded webhook to {}, got {}",
                self.target,
                res.status()
            );

            if res.status().is_client_error() || res.status().is_server_error() {
                println!("{}", res.text().await?);
            }
        }

        Ok(())
    }

    pub fn print_url(&self, client_id: &str, protocol: &str, server: &str) {
        let webhook_url = format!("{}{}/webhook/{}", protocol, server, client_id);
        println!("✅ You can send webhook requests to: {webhook_url}");
    }
}
