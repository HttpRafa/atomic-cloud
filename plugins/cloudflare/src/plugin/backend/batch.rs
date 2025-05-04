use data::{BBatch, BBatchResult};

use crate::error;

use super::Backend;

pub mod data;
pub mod delete;
pub mod record;

impl Backend {
    pub fn send_batch(&self, zone: &str, batch: &BBatch) -> Option<BBatchResult> {
        let response =
            self.post_object_to_api(&format!("zones/{zone}/dns_records/batch"), &batch)?;
        if !response.success {
            error!("Failed to send batched dns updates:");
            for (i, error) in response.errors.into_iter().enumerate() {
                error!("{i}: {:?}", error);
            }
            return None;
        }
        Some(response.result)
    }
}
