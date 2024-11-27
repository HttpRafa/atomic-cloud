use std::fs;

use colored::Colorize;
use common::config::LoadFromTomlFile;
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
            format!("{} deployment(s)", profiles.profiles.len()).blue()
        );
        profiles
    }

    fn add_profile(&mut self, profile: Profile) {
        self.profiles.push(profile);
    }
}

pub struct Profile {
    pub id: String,
    pub name: String,
    pub url: Url,
}

impl Profile {
    fn from(id: &str, profile: &StoredProfile) -> Self {
        Self {
            id: id.to_string(),
            name: profile.name.clone(),
            url: profile.url.clone(),
        }
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

        /* Controller */
        pub url: Url,
    }

    impl LoadFromTomlFile for StoredProfile {}
    impl SaveToTomlFile for StoredProfile {}
}
