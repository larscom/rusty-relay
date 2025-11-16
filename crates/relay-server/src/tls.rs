use crate::util::from_env_or_else;
use axum_server::tls_rustls::RustlsConfig;

pub fn init() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("cryptoprovider should be installed");
}

pub async fn config() -> Option<RustlsConfig> {
    RustlsConfig::from_pem_file(
        from_env_or_else("RUSTY_RELAY_TLS_CERT_FILE", || {
            "./certs/cert.pem".to_string()
        }),
        from_env_or_else("RUSTY_RELAY_TLS_KEY_FILE", || "./certs/key.pem".to_string()),
    )
    .await
    .ok()
}
