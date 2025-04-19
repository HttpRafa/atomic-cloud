use std::fmt::{Display, Formatter};

use ::base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Serialize};

pub mod manager;
pub mod requests;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct KnownHost {
    pub host: String,
    #[serde(with = "base64")]
    pub sha256: Vec<u8>,
}

impl KnownHost {
    pub fn new(host: String, sha256: Vec<u8>) -> Self {
        Self { host, sha256 }
    }
}

impl Display for KnownHost {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", general_purpose::STANDARD.encode(&self.sha256))
    }
}

// Credits: https://users.rust-lang.org/t/serialize-a-vec-u8-to-json-as-base64/57781/2
mod base64 {
    use base64::engine::general_purpose;
    use base64::Engine;
    use serde::{Deserialize, Serialize};
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        let base64 = general_purpose::STANDARD.encode(value);
        String::serialize(&base64, s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let base64 = String::deserialize(d)?;
        general_purpose::STANDARD
            .decode(base64.as_bytes())
            .map_err(serde::de::Error::custom)
    }
}
