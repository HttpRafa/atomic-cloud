use std::process::exit;

use log::error;
use regex::Regex;

pub struct RegexDetector {
    /* Started Regex */
    started_regex: Regex,

    /* Stopping Regex */
    stopping_regex: Option<Regex>,
}

impl RegexDetector {
    pub fn from_env() -> Self {
        let started_regex;
        let stopping_regex;

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

        Self::new(started_regex, stopping_regex)
    }

    pub fn new(started_regex: Regex, stopping_regex: Option<Regex>) -> Self {
        Self {
            started_regex,
            stopping_regex,
        }
    }

    pub fn is_started(&self, line: &str) -> bool {
        return self.started_regex.is_match(line);
    }

    pub fn is_stopping(&self, line: &str) -> bool {
        if let Some(stopping_regex) = &self.stopping_regex {
            return stopping_regex.is_match(line);
        }
        false
    }
}