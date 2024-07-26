use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{Arc, Mutex},
};

use colored::Colorize;
use log::{error, info, warn};
use stored::StoredUser;
use uuid::Uuid;

use crate::config::{LoadFromTomlFile, SaveToTomlFile};

use super::server::{ServerHandle, WeakServerHandle};

const AUTH_DIRECTORY: &str = "auth";
const USERS_DIRECTORY: &str = "users";

const DEFAULT_ADMIN_USERNAME: &str = "admin";

pub type AuthUserHandle = Arc<AuthUser>;
pub type AuthServerHandle = Arc<AuthServer>;

pub struct AuthUser {
    pub username: String,
    pub token: String,
}

pub struct AuthServer {
    pub server: WeakServerHandle,
    pub token: String,
}

pub struct Auth {
    pub users: Mutex<HashMap<String, AuthUserHandle>>,
    pub servers: Mutex<HashMap<String, AuthServerHandle>>,
}

impl Auth {
    pub fn new(users: HashMap<String, AuthUserHandle>) -> Self {
        Auth {
            users: Mutex::new(users),
            servers: Mutex::new(HashMap::new()),
        }
    }

    pub fn load_all() -> Self {
        info!("Loading users...");

        let users_directory = Path::new(AUTH_DIRECTORY).join(USERS_DIRECTORY);
        if !users_directory.exists() {
            if let Err(error) = fs::create_dir_all(&users_directory) {
                warn!("{} to create users directory: {}", "Failed".red(), &error);
            }
        }

        let mut users = HashMap::new();
        let entries = match fs::read_dir(&users_directory) {
            Ok(entries) => entries,
            Err(error) => {
                error!("{} to read users directory: {}", "Failed".red(), &error);
                return Auth::new(users);
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
                .values()
                .any(|u| u.username.eq_ignore_ascii_case(&user.username))
            {
                error!("User with the name {} already exists", &name.red());
                continue;
            }
            users.insert(user.token.clone(), Arc::new(user));
            info!("Loaded user {}", &name.blue());
        }

        let amount = users.len();
        let auth = Auth::new(users);
        if amount == 0 {
            let user = auth
                .register_user(DEFAULT_ADMIN_USERNAME)
                .expect("Failed to create default admin user");
            info!("{}", "-----------------------------------".red());
            info!("{}", "No users found, created default admin user".red());
            info!("{}{}", "Username: ".red(), DEFAULT_ADMIN_USERNAME.red());
            info!("{}{}", "Token: ".red(), &user.token.red());
            info!("{}", "-----------------------------------".red());
            info!(
                "{}",
                "      Welcome to Atomic Cloud       ".bright_blue().bold()
            );
            info!("{}", "-----------------------------------".red());
        }

        info!("Loaded {}", format!("{} user(s)", amount).blue());
        auth
    }

    pub fn get_user(&self, token: &str) -> Option<AuthUserHandle> {
        self.users.lock().unwrap().get(token).cloned()
    }

    pub fn get_server(&self, token: &str) -> Option<AuthServerHandle> {
        self.servers.lock().unwrap().get(token).cloned()
    }

    pub fn register_server(&self, server: WeakServerHandle) -> AuthServerHandle {
        let token = format!(
            "actl_{}{}",
            Uuid::new_v4().as_simple(),
            Uuid::new_v4().as_simple()
        );

        let server = Arc::new(AuthServer {
            server,
            token: token.clone(),
        });
        self.servers
            .lock()
            .unwrap()
            .insert(token.clone(), server.clone());

        server
    }

    pub fn unregister_server(&self, server: &ServerHandle) {
        self.servers.lock().unwrap().retain(|_, value| {
            if let Some(ref_server) = value.server.upgrade() {
                !Arc::ptr_eq(&ref_server, server)
            } else {
                true
            }
        })
    }

    pub fn register_user(&self, username: &str) -> Option<AuthUserHandle> {
        let token = format!(
            "actl_{}{}",
            Uuid::new_v4().as_simple(),
            Uuid::new_v4().as_simple()
        );
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
            return None;
        }

        let user = Arc::new(AuthUser {
            username: username.to_string(),
            token: token.clone(),
        });
        self.users
            .lock()
            .unwrap()
            .insert(token.clone(), user.clone());

        Some(user)
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
