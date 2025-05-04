use std::collections::HashMap;

use anyhow::Result;

use crate::{
    generated::plugin::system::data_types::Uuid,
    plugin::{
        backend::{
            batch::{
                data::{BBatch, BBatchResult},
                delete::BDelete,
                record::BRecord,
            },
            Backend,
        },
        batcher::{Action, Batcher},
        config::{Config, Entry},
    },
};

use super::Record;

#[derive(Default)]
pub struct Zone {
    records: HashMap<Entry, HashMap<Uuid, Record>>,
}

#[derive(Default)]
pub struct Records {
    zones: HashMap<String, Zone>,
}

impl Records {
    pub fn new(config: &Config) -> Self {
        let mut zones: HashMap<String, Zone> = HashMap::new();
        for entry in &config.entries {
            zones
                .entry(entry.zone.clone())
                .or_default()
                .records
                .entry(entry.clone())
                .or_default();
        }
        Self { zones }
    }

    // TODO: Rewrite this i dont like the complexity
    pub fn tick(&mut self, backend: &Backend, batcher: &mut Batcher) -> Result<()> {
        for (zone_id, zone) in &mut self.zones {
            let mut batch = BBatch::default();

            // Process create/delete actions and prepare placeholder records
            if let Some((entry, actions)) = batcher.drain(zone_id) {
                for (uuid, action) in actions.drain() {
                    match action {
                        Action::Create(server) => {
                            // Prepare DNS record and send create
                            let record = Record::new(&entry.weight, server, String::new());
                            if let Some(brecord) = BRecord::new(entry, &record) {
                                batch.posts.push(brecord);
                                // Insert placeholder to be updated once CF returns an ID
                                zone.records
                                    .entry(entry.clone())
                                    .or_default()
                                    .insert(uuid.clone(), record);
                            }
                        }
                        Action::Delete => {
                            // Send delete for existing record
                            if let Some(rec_map) = zone.records.get(entry)
                                && let Some(existing_record) = rec_map.get(&uuid) {
                                    batch.deletes.push(BDelete::from(existing_record));
                                }
                        }
                    }
                }
            }

            // Update weights for all existing records
            for (entry, rec_map) in &mut zone.records {
                for record in rec_map.values_mut() {
                    if record.update(&entry.weight)
                        && let Some(brec) = BRecord::new(entry, record) {
                            batch.patches.push(brec);
                        }
                }
            }

            // Execute batch and apply Cloudflare response
            if let Some(BBatchResult {
                posts,
                patches,
                deletes,
            }) = backend.send_batch(zone_id, batch)
            {
                // Apply deletions
                for delete_result in deletes {
                    if let Some(uuid) = Self::extract_uuid(&delete_result.tags) {
                        for record_map in zone.records.values_mut() {
                            record_map.remove(&uuid);
                        }
                    }
                }

                // Apply creations: set CF-assigned IDs on placeholders
                for created_record in posts {
                    if let Some(uuid) = Self::extract_uuid(&created_record.tags) {
                        for rec_map in zone.records.values_mut() {
                            if let Some(record) = rec_map.get_mut(&uuid) {
                                // Only update placeholders (empty id)
                                if record.id.is_empty() {
                                    record.id = created_record.id.clone();
                                }
                                break;
                            }
                        }
                    }
                }

                // Apply patches: update id (in case CF changed) and weight
                for patch_record in patches {
                    if let Some(uuid) = Self::extract_uuid(&patch_record.tags) {
                        for record_map in zone.records.values_mut() {
                            if let Some(record) = record_map.get_mut(&uuid) {
                                record.id = patch_record.id.clone();
                                record.weight = patch_record.data.weight;
                                break;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn extract_uuid(tags: &[String]) -> Option<String> {
        tags.iter()
            .find_map(|tag| tag.strip_prefix("server:"))
            .map(str::to_string)
    }
}
