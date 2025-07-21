use std::sync::RwLock;

use color_eyre::eyre::Result;
use stored::StoredKnownHosts;

use crate::storage::{LoadFromTomlFile, SaveToTomlFile, Storage};

use super::{
    KnownHost,
    requests::{RequestTracker, TrustRequest, TrustResult},
};

#[derive(Default, Debug)]
pub struct KnownHosts {
    // We cannot use the RwLock from tokio because is_trusted needs to be sync
    // Will be fixed with: https://github.com/rustls/rustls/issues/850
    pub hosts: RwLock<Vec<KnownHost>>,

    pub requests: RequestTracker,
}

impl KnownHosts {
    pub async fn load() -> Result<Self> {
        let file = Storage::known_hosts_file()?;
        if !file.exists() {
            return Ok(KnownHosts::default());
        }

        Ok(Self {
            hosts: RwLock::new(StoredKnownHosts::from_file(&file).await?.hosts),
            requests: RequestTracker::default(),
        })
    }

    pub fn is_trusted(&self, host: &str, sha256: &[u8]) -> TrustResult {
        if let Some(host) = self
            .hosts
            .read()
            .expect("Failed to lock hosts")
            .iter()
            .find(|known| known.host == host && known.sha256 == sha256)
        {
            if host.trusted {
                return TrustResult::Trusted;
            }
            return TrustResult::Declined;
        }

        self.requests.enqueue(TrustRequest::new(KnownHost::new(
            host.to_string(),
            sha256.to_vec(),
        )));
        TrustResult::Declined
    }

    pub async fn set_trust(&self, trusted: bool, request: &mut TrustRequest) -> Result<()> {
        {
            let mut hosts = self.hosts.write().expect("Failed to lock hosts");

            request.complete();
            {
                let mut host = request.get_host().clone();
                host.trusted = trusted;
                hosts.push(host);
            }

            StoredKnownHosts {
                hosts: hosts.clone(),
            }
        }
        .save(&Storage::known_hosts_file()?, true)
        .await?;
        Ok(())
    }
}

mod stored {
    use serde::{Deserialize, Serialize};

    use crate::{
        application::network::known_host::KnownHost,
        storage::{LoadFromTomlFile, SaveToTomlFile},
    };

    #[derive(Serialize, Deserialize)]
    pub struct StoredKnownHosts {
        pub hosts: Vec<KnownHost>,
    }

    impl LoadFromTomlFile for StoredKnownHosts {}
    impl SaveToTomlFile for StoredKnownHosts {}
}
