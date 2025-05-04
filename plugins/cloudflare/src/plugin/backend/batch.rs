use std::collections::hash_map::Drain;

use anyhow::Result;
use data::{BBatch, BBatchResult};
use delete::BDelete;

use crate::{error, plugin::batcher::Action};

use super::{common::data::BObject, Backend};

pub mod data;
pub mod delete;
pub mod record;

impl Backend {
    pub fn send_batch(&self, zone: &str, batch: BBatch) -> Option<BBatchResult> {
        let response =
            self.post_object_to_api(&format!("zones/{}/dns_records/batch", zone), &batch)?;
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
