use std::process::exit;

use regex::Regex;
use simplelog::error;
use uuid::Uuid;

pub struct RegexDetector {
    /* Powerstates Regex */
    started_regex: Regex,
    stopping_regex: Option<Regex>,

    /* User Regex */
    user_connected_regex: Option<Regex>,
    user_disconnected_regex: Option<Regex>,
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

    pub fn new(
        started_regex: Regex,
        stopping_regex: Option<Regex>,
        user_connected_regex: Option<Regex>,
        user_disconnected_regex: Option<Regex>,
    ) -> Self {
        Self {
            started_regex,
            stopping_regex,
            user_connected_regex,
            user_disconnected_regex,
        }
    }

    pub fn detect(&self, line: &str) -> Detection {
        if self.started_regex.is_match(line) {
            return Detection::Started;
        }

        if let Some(stopping_regex) = &self.stopping_regex {
            if stopping_regex.is_match(line) {
                return Detection::Stopping;
            }
        }

        if let Some(user_connected_regex) = &self.user_connected_regex {
            if let Some(captures) = user_connected_regex.captures(line) {
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

        if let Some(user_disconnected_regex) = &self.user_disconnected_regex {
            if let Some(captures) = user_disconnected_regex.captures(line) {
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
