use serde::Serialize;

// SEE: https://developers.cloudflare.com/api/resources/dns/subresources/records/methods/create/

#[derive(Serialize, Clone)]
pub struct BData {
    pub priority: u16,
    pub weight: u16,
    pub port: u16,
    pub target: String,
}

#[derive(Serialize, Clone)]
pub struct BCreate {
    pub comment: String,
    pub data: BData,
    pub name: String,
    pub proxied: bool,
    pub ttl: u16,
    pub r#type: String,
}
