use reqwest::{
    Client, Method,
    header::{HeaderMap, HeaderName, HeaderValue},
};
use rusty_relay_messages::RelayMessage;
use std::{collections::HashMap, str::FromStr};

pub async fn handle_proxy_request(
    request_id: String,
    path: Option<String>,
    method: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
    target: String,
) -> Result<RelayMessage, Box<dyn std::error::Error>> {
    let client = Client::builder().use_rustls_tls().build()?;
    let url = if let Some(p) = path.as_ref() {
        format!("{}/{}", &target, p)
    } else {
        target
    };

    let mut request_headers = HeaderMap::with_capacity(headers.len());
    for (k, v) in headers {
        request_headers.insert(k.parse::<HeaderName>()?, v.parse::<HeaderValue>()?);
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

    Ok(RelayMessage::ProxyResponse {
        request_id,
        body: res.bytes().await?.to_vec(),
        headers: response_headers,
        status,
    })
}
