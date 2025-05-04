use std::collections::HashMap;

use anyhow::Result;

use crate::{
    generated::plugin::system::data_types::Uuid,
    plugin::{
        backend::{
            batch::{
                data::BBatch,
                delete::BDelete,
                record::{BData, BRecord},
            },
            Backend,
        },
        batcher::{Action, Batcher},
        config::{Config, Entry, Weight},
        math::WeightCalc,
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

    pub fn tick(&mut self, backend: &mut Backend, batcher: &mut Batcher) -> Result<()> {
        for (id, zone) in &mut self.zones {
            let mut batch = BBatch::default();
            let mut new = HashMap::new();

            // Loop over all server starts and stops that happend since the last tick
            if let Some((entry, actions)) = batcher.drain(id) {
                for (uuid, action) in actions.drain() {
                    match action {
                        Action::Create(server) => {
                            if let Some(record) = BRecord::new(
                                entry,
                                &Record::new(&entry.weight, server.clone(), String::new()),
                            ) {
                                batch.posts.push(record);
                                new.insert(uuid, (server, entry.clone()));
                            }
                        }
                        Action::Delete => {
                            if let Some(records) = zone.records.get(&entry)
                                && let Some(record) = records.get(&uuid)
                            {
                                batch.deletes.push(BDelete::from(record));
                            }
                        }
                    }
                }
            }

            // Add all updates to the weights
            for (entry, records) in &mut zone.records {
                for (_, record) in records {
                    if record.update(&entry.weight)
                        && let Some(record) = BRecord::new(entry, record)
                    {
                        // Weight has changed
                        batch.patches.push(record);
                    }
                }
            }

            if let Some(result) = backend.send_batch(id, batch) {
                for record in result.deletes {
                    let mut uuid = "";
                    for tag in &record.tags {
                        if tag.starts_with("server:") {
                            let split = tag.split(':').collect::<Vec<_>>();
                            if split.len() >= 2 {
                                uuid = split[1];
                            }
                        }
                    }
                    for (_, records) in &mut zone.records {
                        records.remove(uuid);
                    }
                }
                for record in result.posts {
                    let mut uuid = "";
                    for tag in &record.tags {
                        if tag.starts_with("server:") {
                            let split = tag.split(':').collect::<Vec<_>>();
                            if split.len() >= 2 {
                                uuid = split[1];
                            }
                        }
                    }
                    if let Some((server, entry)) = new.remove(uuid) {
                        zone.records.entry(entry.clone()).or_default().insert(
                            uuid.to_string(),
                            Record::new(&entry.weight, server, record.id),
                        );
                    }
                }
                for record in result.patches {
                    let mut uuid = "";
                    for tag in &record.tags {
                        if tag.starts_with("server:") {
                            let split = tag.split(':').collect::<Vec<_>>();
                            if split.len() >= 2 {
                                uuid = split[1];
                            }
                        }
                    }
                    for (_, records) in &mut zone.records {
                        if let Some(inner) = records.get_mut(uuid) {
                            inner.id = record.id;
                            inner.weight = record.data.weight;
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
