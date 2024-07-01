use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

use colored::Colorize;
use log::{error, info, warn};
use stored::StoredUser;
use uuid::Uuid;

use crate::config::{LoadFromTomlFile, SaveToTomlFile};

use super::server::ServerHandle;

const AUTH_DIRECTORY: &str = "auth";
const USERS_DIRECTORY: &str = "users";

const DEFAULT_ADMIN_USERNAME: &str = "admin";

type AuthUserHandle = Arc<AuthUser>;

pub struct AuthUser {
    pub username: String,
    pub token: String,
}

pub struct AuthServer {
    pub _server: ServerHandle,
    pub _token: String,
}

pub struct Auth {
    pub users: Mutex<Vec<AuthUserHandle>>,
}

impl Auth {
    pub fn load_all() -> Self {
        info!("Loading users...");

        let users_directory = Path::new(AUTH_DIRECTORY).join(USERS_DIRECTORY);
        if !users_directory.exists() {
            if let Err(error) = fs::create_dir_all(&users_directory) {
                warn!("{} to create users directory: {}", "Failed".red(), &error);
            }
        }

        let mut users = Vec::new();
        let entries = match fs::read_dir(&users_directory) {
            Ok(entries) => entries,
            Err(error) => {
                error!("{} to read users directory: {}", "Failed".red(), &error);
                return Auth {
                    users: Mutex::new(users),
                };
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    error!("{} to read user entry: {}", "Failed".red(), &error);
                    continue;
                }
            };

            let path = entry.path();
            if path.is_dir() {
                continue;
            }

            let name = match path.file_stem() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };

            let user = match StoredUser::load_from_file(&path) {
                Ok(user) => user,
                Err(error) => {
                    error!(
                        "{} to read user {} from file({:?}): {}",
                        "Failed".red(),
                        &name,
                        &path,
                        &error
                    );
                    continue;
                }
            };

            let user = AuthUser {
                username: name.clone(),
                token: user.token,
            };
            if users
                .iter()
                .any(|u| u.username.eq_ignore_ascii_case(&user.username))
            {
                error!("User with the name {} already exists", &name.red());
                continue;
            }
            users.push(Arc::new(user));
            info!("Loaded user {}", &name.blue());
        }

        if users.is_empty() {
            let token = format!("actl_{}", Uuid::new_v4().as_simple());
            Self::create_user_in(&mut users, DEFAULT_ADMIN_USERNAME, &token);
            info!("{}", "-----------------------------------".red());
            info!("{}", "No users found, created default admin user".red());
            info!("{}{}", "Username: ".red(), DEFAULT_ADMIN_USERNAME.red());
            info!("{}{}", "Token: ".red(), &token.red());
            info!("{}", "-----------------------------------".red());
            info!("{}", "      Welcome to Atomic Cloud       ".bright_blue().bold());
            info!("{}", "-----------------------------------".red());
        }

        info!("Loaded {}", format!("{} user(s)", users.len()).blue());
        Auth {
            users: Mutex::new(users),
        }
    }

    pub fn get_user(&self, token: &str) -> Option<AuthUserHandle> {
        self.users.lock().unwrap().iter().find(|user| user.token == token).cloned()
    }

    pub fn get_server(&self, _token: &str) -> Option<AuthServer> {
        None
    }

    fn create_user_in(users: &mut Vec<AuthUserHandle>, username: &str, token: &str) -> bool {
        let stored_user = StoredUser {
            token: token.to_string(),
        };
        let user_path = Path::new(AUTH_DIRECTORY)
            .join(USERS_DIRECTORY)
            .join(format!("{}.toml", username));
        if stored_user.save_to_file(&user_path).is_err() {
            error!(
                "{} to save user to file: {}",
                "Failed".red(),
                &user_path.display()
            );
            return false;
        }

        let user = AuthUser {
            username: username.to_string(),
            token: token.to_string(),
        };
        users.push(Arc::new(user));

        true
    }
}

mod stored {
    use crate::config::{LoadFromTomlFile, SaveToTomlFile};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct StoredUser {
        pub token: String,
    }

    impl LoadFromTomlFile for StoredUser {}
    impl SaveToTomlFile for StoredUser {}
}
