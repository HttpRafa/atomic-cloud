use serde::Deserialize;

#[derive(Deserialize)]
pub struct BMetadata {
    pub pagination: BPagination,
}

#[derive(Deserialize)]
pub struct BPagination {
    pub total_pages: u32,
}

#[derive(Deserialize)]
pub struct BBody<T> {
    pub data: T,
    pub meta: BMetadata,
}

#[derive(Deserialize)]
pub struct BObject<T> {
    pub attributes: T
}

pub type BList<T> = BBody<Vec<BObject<T>>>;