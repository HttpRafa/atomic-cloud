use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct TimedName {
    raw_name: String,
    name: String,
}

impl TimedName {
    #[must_use]
    pub fn new(cloud_identifier: &str, name: &str, permanent: bool) -> Self {
        Self {
            raw_name: name.to_string(),
            name: Self::generate(Some(cloud_identifier.to_string()), name, permanent),
        }
    }
    #[must_use]
    pub fn new_no_identifier(name: &str, permanent: bool) -> Self {
        Self {
            raw_name: name.to_string(),
            name: Self::generate(None, name, permanent),
        }
    }

    fn generate(cloud_identifier: Option<String>, name: &str, permanent: bool) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        match (cloud_identifier, permanent) {
            (Some(identifier), true) => format!("{name}@{identifier}"),
            (None, true) => name.to_string(),
            (Some(identifier), false) => format!("{name}@{identifier}#{timestamp}"),
            (None, false) => format!("{name}#{timestamp}"),
        }
    }

    #[must_use]
    pub fn get_name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub fn get_name_cloned(&self) -> String {
        self.name.clone()
    }
    #[must_use]
    pub fn get_raw_name(&self) -> &str {
        &self.raw_name
    }
    #[must_use]
    pub fn get_raw_name_cloned(&self) -> String {
        self.raw_name.clone()
    }
}
