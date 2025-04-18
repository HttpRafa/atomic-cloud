use std::collections::VecDeque;

use color_eyre::eyre::Result;
use stored::StoredKnownHosts;
use tokio::sync::{
    oneshot::{channel, Sender},
    RwLock,
};

use crate::storage::{LoadFromTomlFile, SaveToTomlFile, Storage};

use super::KnownHost;

#[derive(Default, Debug)]
pub struct KnownHosts {
    pub hosts: RwLock<Vec<KnownHost>>,

    pub pending: RwLock<VecDeque<TrustRequest>>,
}

#[derive(Debug)]
pub struct TrustRequest(pub Option<Sender<TrustResult>>, pub KnownHost);

#[derive(Debug)]
pub enum TrustResult {
    Trusted,
    HostDuplicate,
    Declined,
}

impl KnownHosts {
    pub async fn load() -> Result<Self> {
        let file = Storage::known_hosts_file()?;
        if !file.exists() {
            return Ok(KnownHosts::default());
        }

        Ok(Self {
            hosts: RwLock::new(StoredKnownHosts::from_file(&file).await?.hosts),
            pending: RwLock::new(VecDeque::new()),
        })
    }

    pub async fn is_trusted(&self, host: &str, sha256: &[u8]) -> Result<TrustResult> {
        for known in self.hosts.read().await.iter() {
            if known.host == host {
                if known.sha256 == sha256 {
                    return Ok(TrustResult::Trusted);
                }
                return Ok(TrustResult::HostDuplicate);
            }
        }

        let (sender, receiver) = channel();
        self.pending.write().await.push_back(TrustRequest(
            Some(sender),
            KnownHost::new(host.to_string(), sha256.to_vec()),
        ));
        Ok(receiver.await?)
    }

    pub async fn trust(&self, request: &mut TrustRequest) -> Result<()> {
        let mut hosts = self.hosts.write().await;

        request
            .0
            .take()
            .map(|sender| sender.send(TrustResult::Trusted));
        hosts.push(request.1.clone());
        StoredKnownHosts {
            hosts: hosts.clone(),
        }
        .save(&Storage::known_hosts_file()?, true)
        .await?;
        Ok(())
    }

    pub async fn next(&self) -> Option<TrustRequest> {
        self.pending.write().await.pop_front()
    }
}

impl Drop for TrustRequest {
    fn drop(&mut self) {
        self.0
            .take()
            .map(|sender| sender.send(TrustResult::Declined));
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
