use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct BError {
    pub code: u32,
    pub message: String,
    pub documentation_url: String,
}
