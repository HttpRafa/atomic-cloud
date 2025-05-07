use std::collections::HashMap;

use crate::{
    generated::plugin::system::data_types::Uuid,
    info,
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
            if entry.zone.is_empty() {
                continue;
            }
            zones
                .entry(entry.zone.clone())
                .or_default()
                .records
                .entry(entry.clone())
                .or_default();
        }
        info!("Found {} unique zone(s)", zones.len());
        Self { zones }
    }

    pub fn tick(&mut self, backend: &Backend, batcher: &mut Batcher) {
        for (zone_id, zone) in &mut self.zones {
            let mut batch = BBatch::default();

            // Process create/delete actions and prepare placeholder records
            if let Some((entry, actions)) = batcher.drain(zone_id) {
                for (uuid, action) in actions.drain() {
                    match action {
                        Action::Create(server) => {
                            // Prepare DNS record and send create
                            let record = Record::new(&entry.weight, server, None);
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
                            if let Some(record_map) = zone.records.get(entry)
                                && let Some(existing_record) = record_map.get(&uuid)
                            {
                                // If this record is a placeholder we skip it.
                                if existing_record.id.is_none() {
                                    continue;
                                }

                                batch.deletes.push(BDelete::from(existing_record));
                            }
                        }
                    }
                }
            }

            // Update weights for all existing records
            for (entry, record_map) in &mut zone.records {
                for record in record_map.values_mut() {
                    // If this record is a placeholder we skip it.
                    if record.id.is_none() {
                        continue;
                    }

                    if record.update(&entry.weight)
                        && let Some(brec) = BRecord::new(entry, record)
                    {
                        batch.patches.push(brec);
                    }
                }
            }

            if batch.deletes.is_empty()
                && batch.patches.is_empty()
                && batch.posts.is_empty()
                && batch.puts.is_empty()
            {
                // No request to cloudflare required
                continue;
            }

            // Execute batch and apply Cloudflare response
            if let Some(BBatchResult {
                posts,
                patches,
                deletes,
            }) = backend.send_batch(zone_id, &batch)
            {
                // Apply deletions
                if let Some(deletes) = deletes {
                    for delete_result in deletes {
                        for record_map in zone.records.values_mut() {
                            record_map.remove(&delete_result.comment);
                        }
                    }
                }

                // Apply creations: set CF-assigned IDs on placeholders
                if let Some(posts) = posts {
                    for created_record in posts {
                        for record_map in zone.records.values_mut() {
                            if let Some(record) = record_map.get_mut(&created_record.comment) {
                                // Only update placeholders (empty id)
                                if record.id.is_none() {
                                    record.id = Some(created_record.id.clone());
                                }
                                break;
                            }
                        }
                    }
                }

                // Apply patches: update id (in case CF changed) and weight
                if let Some(patches) = patches {
                    for patch_record in patches {
                        for record_map in zone.records.values_mut() {
                            if let Some(record) = record_map.get_mut(&patch_record.comment) {
                                record.weight = patch_record.data.weight;
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn shutdown(&mut self, backend: &Backend) {
        info!("Deleting created records...");

        let mut count = 0;
        for (zone_id, zone) in &mut self.zones {
            let mut batch = BBatch::default();

            for record_map in zone.records.values_mut() {
                for record in record_map.values_mut() {
                    // If this record is a placeholder we skip it.
                    if record.id.is_none() {
                        continue;
                    }

                    batch.deletes.push(BDelete::from(&*record));
                }
            }

            if batch.deletes.is_empty() {
                // No request to cloudflare required
                continue;
            }

            count += batch.deletes.len();
            backend.send_batch(zone_id, &batch);
        }

        info!("Deleted {} records...", count);
    }
}
