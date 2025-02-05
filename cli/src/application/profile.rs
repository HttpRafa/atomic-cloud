use std::{
    fmt::{Display, Formatter},
    fs,
};

use anyhow::{anyhow, Result};
use common::{config::SaveToTomlFile, file::for_each_content_toml};
use simplelog::debug;
use stored::StoredProfile;
use url::Url;

use crate::storage::Storage;

use super::network::{CloudConnection, EstablishedConnection};

pub struct Profiles {
    pub profiles: Vec<Profile>,
}

impl Profiles {
    pub fn init() -> Result<Self> {
        debug!("Loading profiles...");
        let mut profiles = vec![];

        let directory = Storage::profiles_folder();
        if !directory.exists() {
            fs::create_dir_all(&directory)?;
        }

        for (_, _, name, value) in
            for_each_content_toml::<StoredProfile>(&directory, "Failed to read profile from file")?
        {
            debug!("Loaded profile {}", name);
            profiles.push(Profile::from(&name, &value));
        }

        debug!("Loaded {} profile(s)", profiles.len());
        Ok(Self { profiles })
    }

    pub fn create_profile(&mut self, profile: &Profile) -> Result<()> {
        // Check if profile already exists
        if Self::is_id_used(&self.profiles, &profile.id) {
            return Err(anyhow!("Profile '{}' already exists", profile.name));
        }

        let profile = profile.clone();
        profile.mark_dirty()?;
        self.add_profile(profile);
        Ok(())
    }

    pub fn delete_profile(&mut self, profile: &Profile) -> Result<()> {
        let index = self
            .profiles
            .iter()
            .position(|p| p.id == profile.id)
            .ok_or_else(|| anyhow!("Profile '{}' not found", profile.name))?;

        let profile = self.profiles.remove(index);
        profile.delete_file()?;
        Ok(())
    }

    fn add_profile(&mut self, profile: Profile) {
        self.profiles.push(profile);
    }

    pub fn already_exists(profiles: &[Profile], name: &str) -> bool {
        profiles.iter().any(|p| p.id == Profile::compute_id(name))
    }

    pub fn is_id_used(profiles: &[Profile], id: &str) -> bool {
        profiles.iter().any(|p| p.id == id)
    }
}

#[derive(Clone)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub authorization: String,
    pub url: Url,
}

impl Profile {
    pub fn new(name: &str, authorization: &str, url: Url) -> Self {
        Self {
            id: Self::compute_id(name),
            name: name.to_string(),
            authorization: authorization.to_string(),
            url,
        }
    }

    fn from(id: &str, profile: &StoredProfile) -> Self {
        Self {
            id: id.to_string(),
            name: profile.name.clone(),
            authorization: profile.authorization.clone(),
            url: profile.url.clone(),
        }
    }

    pub async fn establish_connection(&self) -> Result<EstablishedConnection> {
        CloudConnection::establish_connection(self).await
    }

    pub fn mark_dirty(&self) -> Result<()> {
        self.save_to_file()
    }

    fn delete_file(&self) -> Result<()> {
        let file_path = Storage::profile_file(&self.id);
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }
        Ok(())
    }

    fn save_to_file(&self) -> Result<()> {
        let stored_profile = StoredProfile {
            name: self.name.clone(),
            authorization: self.authorization.clone(),
            url: self.url.clone(),
        };
        stored_profile.save(&Storage::profile_file(&self.id), true)
    }

    pub fn compute_id(name: &str) -> String {
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

impl Display for Profile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

mod stored {
    use common::config::{LoadFromTomlFile, SaveToTomlFile};
    use serde::{Deserialize, Serialize};
    use url::Url;

    #[derive(Serialize, Deserialize)]
    pub struct StoredProfile {
        /* Settings */
        pub name: String,
        pub authorization: String,

        /* Controller */
        pub url: Url,
    }

    impl LoadFromTomlFile for StoredProfile {}
    impl SaveToTomlFile for StoredProfile {}
}
