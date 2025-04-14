use std::collections::HashMap;

use color_eyre::eyre::{eyre, Result};
use tokio::fs;

use crate::{application::profile::{stored::StoredProfile, Profile}, storage::Storage};

pub struct Profiles {
    pub profiles: HashMap<String, Profile>,
}

impl Profiles {
    pub async fn load() -> Result<Self> {
        let mut profiles = HashMap::new();

        let directory = Storage::profiles_directory()?;
        if !directory.exists() {
            fs::create_dir_all(&directory).await?;
        }

        for (_, _, name, value) in Storage::for_each_content_toml::<StoredProfile>(
            &directory
        )
        .await?
        {
            profiles.insert(name.clone(), Profile::from(&name, &value));
        }

        Ok(Self { profiles })
    }

    pub async fn create_profile(&mut self, profile: &Profile) -> Result<()> {
        // Check if profile already exists
        if self.profiles.contains_key(&profile.id) {
            return Err(eyre!("Profile '{}' already exists", profile.name));
        }

        let profile = profile.clone();
        profile.save_to_file().await?;
        self.profiles.insert(profile.id.clone(), profile);
        Ok(())
    }

    pub async fn delete_profile(&mut self, profile: &Profile) -> Result<()> {
        if let Some(profile) = self.profiles.remove(&profile.id) {
            profile.remove_file().await?;
        }
        Ok(())
    }
}