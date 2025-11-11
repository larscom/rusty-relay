use rustls::pki_types::pem::PemObject;
use std::{fs, sync::Arc};
use tokio_tungstenite::Connector;

pub fn init() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("cryptoprovider should be installed");
}

pub fn connector(ca_cert_path: &Option<String>) -> Option<Connector> {
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
