use std::{collections::HashMap, fs, sync::Arc};

use common::config::{LoadFromTomlFile, SaveToTomlFile};
use simplelog::{error, info, warn};
use stored::StoredUser;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::storage::Storage;

const DEFAULT_ADMIN_USERNAME: &str = "admin";

pub type AuthToken = String;

#[derive(Clone)]
pub enum Authorization {
    User(String), // Username
    Unit(Uuid),   // UUID
}

pub type AuthValidator = Arc<AuthValidatorInner>;

pub struct AuthValidatorInner {
    pub tokens: RwLock<HashMap<AuthToken, Authorization>>,
}

impl AuthValidatorInner {
    pub async fn get_auth(&self, token: &str) -> Option<Authorization> {
        self.tokens.read().await.get(token).cloned()
    }

    pub async fn register_unit(&self, uuid: Uuid) -> String {
        let token = format!(
            "sctl_{}{}",
            Uuid::new_v4().as_simple(),
            Uuid::new_v4().as_simple()
        );

        self.tokens
            .write()
            .await
            .insert(token.clone(), Authorization::Unit(uuid));

        token
    }

    pub async fn unregister(&self, token: &str) {
        self.tokens.write().await.remove(token);
    }

    pub async fn register_user(&self, username: &str) -> Option<String> {
        let token = format!(
            "actl_{}{}",
            Uuid::new_v4().as_simple(),
            Uuid::new_v4().as_simple()
        );
        let stored_user = StoredUser {
            token: token.to_string(),
        };
        let user_path = Storage::get_user_file(username);
        if stored_user.save_to_file(&user_path, true).is_err() {
            error!(
                "<red>Failed</> to save user to file: <red>{}</>",
                &user_path.display()
            );
            return None;
        }
        self.tokens.write().await.insert(token.clone(), Authorization::User(username.to_string()));

        Some(token)
    }
}

pub struct Auth {
    pub validator: AuthValidator,
}

impl Auth {
    pub fn new(users: HashMap<AuthToken, Authorization>) -> Self {
        Auth {
            validator: Arc::new(AuthValidatorInner {
                tokens: RwLock::new(users),
            }),
        }
    }

    pub async fn load_all() -> Self {
        info!("Loading users...");

        let users_directory = Storage::get_users_folder();
        if !users_directory.exists() {
            if let Err(error) = fs::create_dir_all(&users_directory) {
                warn!(
                    "<red>Failed</> to create users directory: <red>{}</>",
                    &error
                );
            }
        }

        let mut users = HashMap::new();
        let entries = match fs::read_dir(&users_directory) {
            Ok(entries) => entries,
            Err(error) => {
                error!("<red>Failed</> to read users directory: <red>{}</>", &error);
                return Auth::new(users);
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    error!("<red>Failed</> to read user entry: <red>{}</>", &error);
                    continue;
                }
            };

            let path = entry.path();
            if path.is_dir() {
                continue;
            }

            let username = match path.file_stem() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };

            let user = match StoredUser::load_from_file(&path) {
                Ok(user) => user,
                Err(error) => {
                    error!(
                        "<red>Failed</> to read user <blue>{}</> from file(<blue>{:?}</>): <red>{}</>",
                        &username,
                        &path,
                        &error
                    );
                    continue;
                }
            };

            if users
                .values()
                .filter_map(|entry| match entry {
                    Authorization::User(name) => Some(name),
                    _ => None,
                })
                .any(|name| name.eq_ignore_ascii_case(&username))
            {
                error!("User with the name <red>{}</> already exists", &username);
                continue;
            }
            info!("Loaded user <blue>{}</>", &username);
            users.insert(user.token.clone(), Authorization::User(username));
        }

        let amount = users.len();
        let auth = Auth::new(users);
        if amount == 0 {
            let token = auth.get_validator()
                .register_user(DEFAULT_ADMIN_USERNAME).await
                .expect("Failed to create default admin user");
            info!("<red>-----------------------------------</>");
            info!("<red>No users found, created default admin user</>");
            info!("<red>Username: </>{}", DEFAULT_ADMIN_USERNAME);
            info!("<red>Token: </>{}", &token);
            info!("<red>-----------------------------------</>");
            info!("<bright-blue><b>Welcome to Atomic Cloud</>");
            info!("<red>-----------------------------------</>");
        }

        info!("Loaded <blue>{} user(s)</>", amount);
        auth
    }

    pub fn get_validator(&self) -> AuthValidator {
        self.validator.clone()
    }
}

mod stored {
    use common::config::{LoadFromTomlFile, SaveToTomlFile};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct StoredUser {
        pub token: String,
    }

    impl LoadFromTomlFile for StoredUser {}
    impl SaveToTomlFile for StoredUser {}
}
