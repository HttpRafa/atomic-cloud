use serde::Deserialize;

use super::error::BError;

#[derive(Deserialize)]
pub struct BObject<T> {
    pub errors: Vec<BError>,
    pub success: bool,
    pub result: T,
}
