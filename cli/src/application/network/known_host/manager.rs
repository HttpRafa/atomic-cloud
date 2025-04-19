use std::sync::RwLock;

use color_eyre::eyre::Result;
use stored::StoredKnownHosts;

use crate::storage::{LoadFromTomlFile, SaveToTomlFile, Storage};

use super::{
    requests::{RequestTracker, TrustRequest, TrustResult},
    KnownHost,
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
        for known in self.hosts.read().unwrap().iter() {
            if known.host == host {
                if known.sha256 == sha256 {
                    return TrustResult::Trusted;
                }
                return TrustResult::HostDuplicate;
            }
        }

        self.requests.enqueue(TrustRequest::new(KnownHost::new(
            host.to_string(),
            sha256.to_vec(),
        )));
        TrustResult::Declined
    }

    pub async fn trust(&self, request: &mut TrustRequest) -> Result<()> {
        {
            let mut hosts = self.hosts.write().unwrap();

            request.complete();
            hosts.push(request.get_host().clone());

            StoredKnownHosts {
                hosts: hosts.clone(),
            }
        }
        .save(&Storage::known_hosts_file()?, true)
        .await?;
        Ok(())
    }

    pub fn get_requests(&self) -> &RequestTracker {
        &self.requests
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
