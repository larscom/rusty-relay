use reqwest::{
    Client, Method,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use rusty_relay_messages::RelayMessage;
use std::{collections::HashMap, str::FromStr};

use crate::error;

#[derive(Debug)]
pub struct ProxyHandler<'a> {
    target: &'a str,
    http_client: Client,
}

impl<'a> ProxyHandler<'a> {
    pub fn new(target: &'a str, http_client: Client) -> Self {
        Self {
            target,
            http_client,
        }
    }

    pub async fn handle(
        &self,
        request_id: String,
        path: Option<String>,
        method: String,
        headers: HashMap<String, String>,
        body: Vec<u8>,
    ) -> Result<Option<RelayMessage>, error::Error> {
        let url = if let Some(p) = path.as_ref() {
            &format!("{}/{}", self.target, p)
        } else {
            self.target
        };

        let mut request_headers = HeaderMap::with_capacity(headers.len());
        for (k, v) in headers {
            request_headers.insert(k.parse::<HeaderName>()?, v.parse::<HeaderValue>()?);
        }

        let response = self
            .http_client
            .request(Method::from_str(&method)?, url)
            .headers(request_headers)
            .body(body)
            .send()
            .await
            .map_err(|err| println!("⚠️ WARNING: request to {} failed: {err}", &self.target));

        if let Ok(res) = response {
            let mut response_headers = HashMap::new();
            for (k, v) in res.headers() {
                if let Ok(value) = v.to_str() {
                    response_headers.insert(k.to_string(), value.to_string());
                }
            }
            let status = res.status().as_u16();

            return Ok(Some(RelayMessage::ProxyResponse {
                request_id,
                body: res.bytes().await?.to_vec(),
                headers: response_headers,
                status,
            }));
        }

        Ok(None)
    }

    pub fn print_url(&self, client_id: &str, protocol: &str, server: &str) {
        let proxy_url = format!("{}{}/proxy/{}", protocol, server, client_id);
        println!("✅ You can send proxy requests to: {proxy_url}")
    }
}
