use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct BObject<T> {
    pub success: bool,
    pub result: T,
}
