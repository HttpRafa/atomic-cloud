use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct BError {
    pub code: u32,
    pub message: String,
    pub documentation_url: String,
}
