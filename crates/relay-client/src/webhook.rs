use reqwest::Client;

pub async fn forward(target: &str, payload: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder().use_rustls_tls().build()?;
    let res = client.post(target).body(payload.to_string()).send().await?;
    println!("➡️ forwarded webhook to {}, got {}", target, res.status());
    if res.status().is_client_error() || res.status().is_server_error() {
        println!("{}", res.text().await?)
    }
    Ok(())
}
