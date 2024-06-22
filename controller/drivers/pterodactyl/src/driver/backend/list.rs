use serde::Deserialize;

#[derive(Deserialize)]
pub struct BList<T> {
    pub data: Vec<BObject<T>>,
}

#[derive(Deserialize)]
pub struct BObject<T> {
    pub attributes: T
}