//! Port allocation. The set of allocated ports lives in the kv store and is
//! updated atomically, so concurrent `allocate` calls never hand out the
//! same port twice.

use anyhow::{Result, bail};

use crate::{
    config::Config,
    generated::atomic::plugin_system::{errors::Error, types::Address},
    state::{self, PORTS_KEY, PortsRecord},
};

pub fn allocate(config: &Config, server: &str, required: u32) -> Result<Vec<Address>> {
    let required = required as usize;
    let host = config.host().to_string();

    // Ports mapped to this server name in the config are used first
    let mapped: Vec<u16> = config
        .mappings()
        .get(server)
        .map_or_else(Vec::new, |ports| {
            ports.iter().take(required).copied().collect()
        });

    // Mapped ports are reserved for their server, even when they fall into
    // the range, so no other server can take them
    let reserved: Vec<u16> = config.mappings().values().flatten().copied().collect();

    let ports = state::update(PORTS_KEY, |record: Option<PortsRecord>| {
        let mut record = record.unwrap_or_default();
        for port in &mapped {
            if record.allocated.contains(port) {
                bail!(Error::AlreadyExists(format!(
                    "Port {port} mapped to {server} is still in use"
                )));
            }
            record.allocated.push(*port);
        }
        let mut ports = mapped.clone();
        let mut candidates = config.range().clone();
        while ports.len() < required {
            let Some(port) = candidates
                .find(|port| !record.allocated.contains(port) && !reserved.contains(port))
            else {
                bail!(Error::NotReady(format!(
                    "No free port left in range {:?}",
                    config.range()
                )));
            };
            record.allocated.push(port);
            ports.push(port);
        }
        record.allocated.sort_unstable();
        Ok((Some(record), ports))
    })?;

    Ok(ports
        .into_iter()
        .map(|port| Address {
            host: host.clone(),
            port,
        })
        .collect())
}

pub fn free(addresses: &[Address]) -> Result<()> {
    state::update(PORTS_KEY, |record: Option<PortsRecord>| {
        let mut record = record.unwrap_or_default();
        record
            .allocated
            .retain(|port| !addresses.iter().any(|address| address.port == *port));
        Ok((Some(record), ()))
    })
}
