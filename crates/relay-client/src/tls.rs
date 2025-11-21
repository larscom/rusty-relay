use anyhow::{Context, anyhow};
use rustls::pki_types::pem::PemObject;
use std::{fs, sync::Arc};
use tokio_tungstenite::Connector;

pub fn init() -> anyhow::Result<()> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .map_err(|_| anyhow!("failed to install default crypto provider"))?;
    Ok(())
}

pub fn connector(ca_cert_path: &Option<String>) -> anyhow::Result<Option<Connector>> {
    let Some(cert_path) = ca_cert_path.as_ref() else {
        return Ok(None);
    };

    let ca_cert = fs::read(cert_path)
        .with_context(|| format!("failed to read CA certificate at path: {}", cert_path))?;

    let pem = rustls::pki_types::CertificateDer::from_pem_slice(&ca_cert)
        .context("CA certificate is not valid PEM")?;

    let mut root_store = rustls::RootCertStore::empty();
    root_store
        .add(pem)
        .context("failed to add CA certificate to RootCertStore")?;

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    Ok(Some(Connector::Rustls(Arc::new(config))))
}
