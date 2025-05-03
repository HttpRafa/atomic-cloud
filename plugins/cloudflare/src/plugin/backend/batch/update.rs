use serde::Serialize;

use super::create::BData;

// SEE: https://developers.cloudflare.com/api/resources/dns/subresources/records/methods/update/

#[derive(Serialize, Clone)]
pub struct BUpdate {
    pub comment: String,
    pub data: BData,
    pub name: String,
    pub proxied: bool,
    pub ttl: u16,
    pub r#type: String,
}
