use std::{
    fmt::{Display, Formatter},
    fs,
};

use anyhow::{anyhow, Result};
use colored::Colorize;
use common::config::{LoadFromTomlFile, SaveToTomlFile};
use log::{info, warn};
use stored::StoredProfile;
use url::Url;

use crate::storage::Storage;

pub struct Profiles {
    pub profiles: Vec<Profile>,
}

impl Profiles {
    fn new() -> Self {
        Profiles { profiles: vec![] }
    }

    pub fn load_all() -> Self {
        info!("Loading profiles...");

        let mut profiles = Self::new();
        let profiles_directory = Storage::get_profiles_folder();
        if !profiles_directory.exists() {
            if let Err(error) = fs::create_dir_all(&profiles_directory) {
                warn!(
                    "{} to create deployments directory: {}",
                    "Failed".red(),
                    &error
                );
                return profiles;
            }
        }

        let entries = match fs::read_dir(&profiles_directory) {
            Ok(entries) => entries,
            Err(error) => {
                warn!(
                    "{} to read deployments directory: {}",
                    "Failed".red(),
                    &error
                );
                return profiles;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    warn!(
                        "{} to read entry in profiles directory: {}",
                        "Failed".red(),
                        &error
                    );
                    continue;
                }
            };

            let path = entry.path();
            if path.is_dir() {
                continue;
            }

            let id: String = match path.file_stem() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };

            let profile = match StoredProfile::load_from_file(&path) {
                Ok(profile) => profile,
                Err(error) => {
                    warn!(
                        "{} to load profile from file '{}': {}",
                        "Failed".red(),
                        path.display(),
                        &error
                    );
                    continue;
                }
            };

            info!("Loading profile '{}'", id.blue());
            let profile = Profile::from(&id, &profile);

            profiles.add_profile(profile);
        }

        info!(
            "Loaded {}",
            format!("{} profiles(s)", profiles.profiles.len()).blue()
        );
        profiles
    }

    pub fn create_profile(&mut self, profile: &Profile) -> Result<()> {
        // Check if profile already exists
        if self.profiles.iter().any(|p| p.id == profile.id) {
            return Err(anyhow!("Profile '{}' already exists", profile.name));
        }

        let profile = profile.clone();
        profile.mark_dirty()?;
        self.add_profile(profile);
        Ok(())
    }

    fn add_profile(&mut self, profile: Profile) {
        self.profiles.push(profile);
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

    fn compute_id(name: &str) -> String {
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

    pub fn mark_dirty(&self) -> Result<()> {
        self.save_to_file()
    }

    fn _delete_file(&self) -> Result<()> {
        let file_path = Storage::get_profile_file(&self.id);
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
        stored_profile.save_to_file(&Storage::get_profile_file(&self.id))
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
