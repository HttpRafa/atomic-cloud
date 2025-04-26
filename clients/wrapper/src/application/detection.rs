use std::process::exit;

use regex::Regex;
use simplelog::error;
use uuid::Uuid;

pub struct RegexDetector {
    /* Powerstates Regex */
    started: Regex,
    stopping: Option<Regex>,

    /* User Regex */
    user_connected: Option<Regex>,
    user_disconnected: Option<Regex>,
}

pub struct DetectedUser {
    pub name: Option<String>,
    pub uuid: Option<Uuid>,
}

pub enum Detection {
    None,
    Started,
    Stopping,
    UserConnected(DetectedUser),
    UserDisconnected(DetectedUser),
}

impl RegexDetector {
    pub fn from_env() -> Self {
        // Powerstates
        let started_regex;
        let stopping_regex;

        // User
        let user_connected_regex;
        let user_disconnected_regex;

        if let Ok(value) = std::env::var("STARTED_REGEX") {
            if let Ok(value) = Regex::new(&value) {
                started_regex = value;
            } else {
                error!("Failed to parse STARTED_REGEX environment variable");
                exit(1);
            }
        } else {
            error!("Missing STARTED_REGEX environment variable. Please set it to the regex that indicates the process has started");
            exit(1);
        }

        if let Ok(value) = std::env::var("STOPPING_REGEX") {
            if let Ok(value) = Regex::new(&value) {
                stopping_regex = Some(value);
            } else {
                error!("Failed to parse STOPPING_REGEX environment variable");
                exit(1);
            }
        } else {
            stopping_regex = None;
        }

        if let Ok(value) = std::env::var("USER_CONNECTED_REGEX") {
            if let Ok(value) = Regex::new(&value) {
                user_connected_regex = Some(value);
            } else {
                error!("Failed to parse USER_CONNECTED_REGEX environment variable");
                exit(1);
            }
        } else {
            user_connected_regex = None;
        }

        if let Ok(value) = std::env::var("USER_DISCONNECTED_REGEX") {
            if let Ok(value) = Regex::new(&value) {
                user_disconnected_regex = Some(value);
            } else {
                error!("Failed to parse USER_DISCONNECTED_REGEX environment variable");
                exit(1);
            }
        } else {
            user_disconnected_regex = None;
        }

        Self::new(
            started_regex,
            stopping_regex,
            user_connected_regex,
            user_disconnected_regex,
        )
    }

    pub const fn new(
        started: Regex,
        stopping: Option<Regex>,
        user_connected: Option<Regex>,
        user_disconnected: Option<Regex>,
    ) -> Self {
        Self {
            started,
            stopping,
            user_connected,
            user_disconnected,
        }
    }

    pub fn detect(&self, line: T) where T: Into<Cow<'a, str>> -> Detection {
        if self.started.is_match(line) {
            return Detection::Started;
        }

        if let Some(stopping) = &self.stopping {
            if stopping.is_match(line) {
                return Detection::Stopping;
            }
        }

        if let Some(user_connected) = &self.user_connected {
            if let Some(captures) = user_connected.captures(line) {
                if let (Some(name), Some(uuid)) = (captures.get(1), captures.get(2)) {
                    if let Ok(parsed_uuid) = Uuid::parse_str(uuid.as_str()) {
                        return Detection::UserConnected(DetectedUser {
                            name: Some(name.as_str().to_string()),
                            uuid: Some(parsed_uuid),
                        });
                    }
                }
            }
        }

        if let Some(user_disconnected) = &self.user_disconnected {
            if let Some(captures) = user_disconnected.captures(line) {
                if let Some(name) = captures.get(1) {
                    return Detection::UserDisconnected(DetectedUser {
                        name: Some(name.as_str().to_string()),
                        uuid: None,
                    });
                }
            }
        }

        Detection::None
    }
}
