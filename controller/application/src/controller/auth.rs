use std::{
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHashString, SaltString},
    Argon2, PasswordHasher, PasswordVerifier,
};
use colored::Colorize;
use log::{error, info, warn};
use once_cell::sync::Lazy;
use stored::StoredUser;
use uuid::Uuid;

use crate::config::{LoadFromTomlFile, SaveToTomlFile};

use super::server::ServerHandle;

const AUTH_DIRECTORY: &str = "auth";
const USERS_DIRECTORY: &str = "users";

const DEFAULT_ADMIN_USERNAME: &str = "admin";

static ARGON2: Lazy<Argon2> = Lazy::new(Argon2::default);

type AuthUserHandle = Arc<AuthUser>;

pub struct AuthUser {
    pub username: String,
    pub token: PasswordHashString,
}

pub struct AuthServer {
    pub _server: ServerHandle,
    pub _token: PasswordHashString,
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

            let token = match user.parse_argon2() {
                Ok(hash) => hash,
                Err(error) => {
                    error!(
                        "{} to parse user token {} from file({:?}): {}",
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
                token,
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
            let token = Uuid::new_v4().to_string();
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
        self.users.lock().unwrap().iter().find_map(|user| {
            ARGON2
                .verify_password(token.as_bytes(), &user.token.password_hash())
                .ok()
                .map(|_| user.clone())
        })
    }

    pub fn get_server(&self, _token: &str) -> Option<AuthServer> {
        None
    }

    fn create_user_in(users: &mut Vec<AuthUserHandle>, username: &str, token: &str) -> bool {
        let salt = SaltString::generate(&mut OsRng);
        let token = match ARGON2.hash_password(token.as_bytes(), &salt) {
            Ok(token) => token,
            Err(error) => {
                error!("{} to hash token: {}", "Failed".red(), &error);
                return false;
            }
        };

        let stored_user = StoredUser {
            token: token.serialize().to_string(),
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
            token: token.serialize(),
        };
        users.push(Arc::new(user));

        true
    }
}

mod stored {
    use crate::config::{LoadFromTomlFile, SaveToTomlFile};
    use argon2::password_hash::{PasswordHashString, Result};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    pub struct StoredUser {
        pub token: String,
    }

    impl StoredUser {
        pub fn parse_argon2(self) -> Result<PasswordHashString> {
            PasswordHashString::new(&self.token)
        }
    }

    impl LoadFromTomlFile for StoredUser {}
    impl SaveToTomlFile for StoredUser {}
}
