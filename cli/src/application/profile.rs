use color_eyre::eyre::Result;
use stored::StoredProfile;
use tokio::fs;
use url::Url;

use crate::storage::{SaveToTomlFile, Storage};

pub mod manager;

#[derive(Clone)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub token: String,
    pub url: Url,

    pub certificate: Option<String>,
}

impl Profile {
    pub fn new(name: &str, token: &str, url: Url, certificate: Option<String>) -> Self {
        Self {
            id: Self::create_id(name),
            name: name.to_string(),
            token: token.to_string(),
            url,
            certificate,
        }
    }

    fn from(id: &str, profile: &StoredProfile) -> Self {
        Self {
            id: id.to_string(),
            name: profile.name.clone(),
            token: profile.token.clone(),
            url: profile.url.clone(),
            certificate: profile.certificate.clone(),
        }
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
            certificate: self.certificate.clone(),
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
        pub certificate: Option<String>,
    }

    impl LoadFromTomlFile for StoredProfile {}
    impl SaveToTomlFile for StoredProfile {}
}
