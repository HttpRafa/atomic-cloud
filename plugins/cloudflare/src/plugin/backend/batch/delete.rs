use serde::Serialize;

// SEE: https://developers.cloudflare.com/api/resources/dns/subresources/records/methods/delete/

#[derive(Serialize, Clone)]
pub struct BDelete {
    id: String,
}
