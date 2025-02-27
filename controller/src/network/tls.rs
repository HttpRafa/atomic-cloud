use anyhow::Result;
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair};
use simplelog::info;
use tokio::fs;
use tonic::transport::Identity;

use crate::storage::Storage;

pub struct Tls;

impl Tls {
    pub async fn load_server_identity(alt_names: &[String]) -> Result<(String, Identity)> {
        let directory = Storage::cert_directory();
        if !directory.exists() {
            fs::create_dir_all(&directory).await?;
        }

        let cert = Storage::cert_file();
        let private_key = Storage::cert_private_key_file();

        if !cert.exists() || !private_key.exists() {
            Self::generate_cert(alt_names).await?;
        }

        let cert = fs::read(&cert).await?;
        let private_key = fs::read(&private_key).await?;

        Ok((
            String::from_utf8(cert.clone())?,
            Identity::from_pem(cert, private_key),
        ))
    }

    pub async fn generate_cert(alt_names: &[String]) -> Result<()> {
        info!("Generating self-signed certificate...");
        let key_pair = KeyPair::generate()?;
        let mut params = CertificateParams::new(alt_names)?;
        params.distinguished_name = DistinguishedName::new();
        params
            .distinguished_name
            .push(DnType::CommonName, "AtomicCloud Self-Signed Cert");
        let cert = params.self_signed(&key_pair)?;
        let cert_pem = cert.pem();
        let private_key_pem = key_pair.serialize_pem();

        fs::write(&Storage::cert_file(), cert_pem).await?;
        fs::write(&Storage::cert_private_key_file(), private_key_pem).await?;
        Ok(())
    }
}
