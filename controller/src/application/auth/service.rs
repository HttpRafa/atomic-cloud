use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use common::{config::SaveToTomlFile, file::for_each_content_toml};
use simplelog::{error, info};
use stored::StoredUser;
use tokio::{fs, sync::RwLock};
use uuid::Uuid;

use crate::storage::Storage;

use super::{AdminUser, AuthToken, Authorization};

pub struct AuthService {
    tokens: RwLock<HashMap<AuthToken, Authorization>>,
}

impl AuthService {
    pub async fn init() -> Result<Arc<AuthService>> {
        info!("Loading users...");
        let mut tokens = HashMap::new();

        let directory = Storage::users_directory();
        if !directory.exists() {
            fs::create_dir_all(&directory).await?;
        }

        for (_, _, name, value) in
            for_each_content_toml::<StoredUser>(&directory, "Failed to read user from file")?
        {
            info!("Loaded user {}", name);
            tokens.insert(
                value.token().clone(),
                Authorization::User(AdminUser::new(name)),
            );
        }

        info!("Loaded {} user(s)", tokens.len());
        Ok(Arc::new(Self {
            tokens: RwLock::new(tokens),
        }))
    }

    pub async fn has_access(&self, token: &str) -> Option<Authorization> {
        self.tokens.read().await.get(token).cloned()
    }

    pub async fn unregister(&self, token: &str) {
        self.tokens.write().await.remove(token);
    }

    pub async fn register_server(&self, uuid: Uuid) -> String {
        let token = format!(
            "sctl_{}{}",
            Uuid::new_v4().as_simple(),
            Uuid::new_v4().as_simple()
        );

        self.tokens
            .write()
            .await
            .insert(token.clone(), Authorization::Server(uuid));

        token
    }

    pub async fn register_user(&self, username: &str) -> Option<String> {
        let token = format!(
            "actl_{}{}",
            Uuid::new_v4().as_simple(),
            Uuid::new_v4().as_simple()
        );
        if let Err(error) = StoredUser::new(&token).save(&Storage::user_file(username), true) {
            error!("Failed to save user({}) to file: {}", username, error);
            return None;
        }
        self.tokens.write().await.insert(
            token.clone(),
            Authorization::User(AdminUser::new(username.to_string())),
        );

        Some(token)
    }
}

mod stored {
    use common::config::{LoadFromTomlFile, SaveToTomlFile};
    use getset::Getters;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Getters)]
    pub struct StoredUser {
        #[getset(get = "pub")]
        token: String,
    }

    impl StoredUser {
        pub fn new(token: &str) -> Self {
            Self {
                token: token.to_string(),
            }
        }
    }

    impl LoadFromTomlFile for StoredUser {}
    impl SaveToTomlFile for StoredUser {}
}
