use std::sync::Arc;

use color_eyre::eyre::Result;
use stored::StoredProfile;
use tokio::fs;
use url::Url;

use crate::storage::{SaveToTomlFile, Storage};

use super::network::{
    connection::{task::ConnectTask, EstablishedConnection},
    known_host::manager::KnownHosts,
};

pub mod manager;

#[derive(Clone)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub token: String,
    pub url: Url,
}

impl Profile {
    pub fn new(name: &str, token: &str, url: Url) -> Self {
        Self {
            id: Self::create_id(name),
            name: name.to_string(),
            token: token.to_string(),
            url,
        }
    }

    fn from(id: &str, profile: &StoredProfile) -> Self {
        Self {
            id: id.to_string(),
            name: profile.name.clone(),
            token: profile.token.clone(),
            url: profile.url.clone(),
        }
    }

    pub fn establish_connection(&self, known_hosts: Arc<KnownHosts>) -> ConnectTask {
        EstablishedConnection::establish_new(
            self.name.clone(),
            self.url.clone(),
            self.token.clone(),
            known_hosts,
        )
    }

    async fn remove_file(&self) -> Result<()> {
        let file = Storage::profile_file(&self.id)?;
        if file.exists() {
            fs::remove_file(&file).await?;
        }
        Ok(())
    }

    async fn save_to_file(&self) -> Result<()> {
        let profile = StoredProfile {
            name: self.name.clone(),
            token: self.token.clone(),
            url: self.url.clone(),
        };
        profile
            .save(&Storage::profile_file(&self.id)?, true)
            .await?;
        Ok(())
    }

    pub fn create_id(name: &str) -> String {
        name.chars()
            .map(|c| match c {
                '/' | ':' | '|' => '-',
                '<' | '>' | '"' | '\\' | '?' | '*' => '.',
                ' ' => '_',
                _ => c,
            })
            .collect::<String>()
            .to_lowercase()
    }
}

mod stored {
    use serde::{Deserialize, Serialize};
    use url::Url;

    use crate::storage::{LoadFromTomlFile, SaveToTomlFile};

    #[derive(Serialize, Deserialize)]
    pub struct StoredProfile {
        pub name: String,
        pub token: String,
        pub url: Url,
    }

    impl LoadFromTomlFile for StoredProfile {}
    impl SaveToTomlFile for StoredProfile {}
}
