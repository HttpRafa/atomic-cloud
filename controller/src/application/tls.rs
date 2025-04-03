use anyhow::Result;
use simplelog::{info, warn};
use tonic::transport::Identity;

use crate::{config::Config, network::tls::Tls};

pub struct TlsSetting {
    pub tls: Option<(String, Identity)>,
}

impl TlsSetting {
    pub async fn init(config: &Config) -> Result<Self> {
        let tls = if config.tls_enabled() {
            info!("Loading TLS certificate...");
            Some(Tls::load_server_identity(config.tls_alt_names()).await?)
        } else {
            warn!("TLS is disabled. NOTE: This is not recommended for production use if you dont have a reverse proxy in front of the controller.");
            None
        };
        Ok(Self { tls })
    }
}
