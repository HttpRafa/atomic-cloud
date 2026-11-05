//! Typed access to the host kv store.
//!
//! Values are TOML documents. Several calls may run at the same time (for
//! example a node tick while the controller stops a server), so every
//! read-modify-write goes through [`update`] or [`swap`], which only commit
//! if nobody changed the key in the meantime.

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Result, anyhow, bail};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::generated::atomic::plugin_system::{errors::Error, kv};

/// How often [`update`] retries before giving up on a contended key.
const MAX_ATTEMPTS: usize = 64;

/* Keys */
pub const PORTS_KEY: &str = "ports";
pub const SERVER_PREFIX: &str = "server/";
pub const TEMPLATE_PREFIX: &str = "template/";

pub fn node_key(node: &str) -> String {
    format!("node/{node}")
}
pub fn server_prefix(node: &str) -> String {
    format!("{SERVER_PREFIX}{node}/")
}
pub fn server_key(node: &str, uuid: &str) -> String {
    format!("{SERVER_PREFIX}{node}/{uuid}")
}
pub fn template_key(name: &str) -> String {
    format!("{TEMPLATE_PREFIX}{name}")
}

/* Records */
#[derive(Serialize, Deserialize)]
pub struct NodeRecord {
    pub controller: String,
}

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
pub struct PortsRecord {
    pub allocated: Vec<u16>,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum TemplateStatus {
    Preparing,
    Ready,
    Failed,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TemplateRecord {
    pub status: TemplateStatus,
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ServerState {
    Running,
    Restarting,
    Stopping,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerRecord {
    pub uuid: String,
    pub name: String,
    pub template: String,
    /// Directory of the server, relative to the data directory.
    pub directory: String,
    pub temporary: bool,
    pub state: ServerState,
    /// Unix timestamp (seconds) of the last state change.
    pub since: u64,
}

/* Access */
pub fn get<T: DeserializeOwned>(key: &str) -> Result<Option<T>> {
    kv::get(key)
        .map_err(|error| anyhow!(error))?
        .map(|bytes| decode(key, &bytes))
        .transpose()
}

pub fn put<T: Serialize>(key: &str, value: &T) -> Result<()> {
    kv::set(key, &encode(value)?).map_err(|error| anyhow!(error))
}

pub fn delete(key: &str) -> Result<()> {
    kv::delete(key).map_err(|error| anyhow!(error))
}

/// A value together with the exact bytes it was decoded from, so it can be
/// written back with [`swap`] only if it is unchanged.
pub struct Versioned<T> {
    pub key: String,
    pub raw: Vec<u8>,
    pub value: T,
}

/// All values whose key starts with `prefix`.
pub fn list<T: DeserializeOwned>(prefix: &str) -> Result<Vec<Versioned<T>>> {
    let mut values = vec![];
    for key in kv::keys(prefix).map_err(|error| anyhow!(error))? {
        if let Some(raw) = kv::get(&key).map_err(|error| anyhow!(error))? {
            let value = decode(&key, &raw)?;
            values.push(Versioned { key, raw, value });
        }
    }
    Ok(values)
}

/// Replaces `current` with `new` (or deletes it for `None`) if nobody
/// changed it since it was read. Returns whether the write happened.
pub fn swap<T: Serialize>(current: &Versioned<impl Sized>, new: Option<&T>) -> Result<bool> {
    let new = new.map(encode).transpose()?;
    kv::compare_and_swap(&current.key, Some(&current.raw), new.as_deref())
        .map_err(|error| anyhow!(error))
}

/// Atomically applies `change` to the value at `key`. `change` may run more
/// than once if the key is modified concurrently, so it must not have side
/// effects. Returning `None` deletes the key.
pub fn update<T, R, F>(key: &str, mut change: F) -> Result<R>
where
    T: Serialize + DeserializeOwned,
    F: FnMut(Option<T>) -> Result<(Option<T>, R)>,
{
    for _ in 0..MAX_ATTEMPTS {
        let raw = kv::get(key).map_err(|error| anyhow!(error))?;
        let current = raw.as_deref().map(|bytes| decode(key, bytes)).transpose()?;
        let (new, result) = change(current)?;
        let new = new.as_ref().map(encode).transpose()?;
        if kv::compare_and_swap(key, raw.as_deref(), new.as_deref())
            .map_err(|error| anyhow!(error))?
        {
            return Ok(result);
        }
    }
    bail!(Error::KeyValue(format!(
        "Key {key} kept changing during {MAX_ATTEMPTS} attempts"
    )))
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    Ok(toml::to_string(value)?.into_bytes())
}

fn decode<T: DeserializeOwned>(key: &str, bytes: &[u8]) -> Result<T> {
    let text = std::str::from_utf8(bytes)?;
    toml::from_str(text).map_err(|error| anyhow!("Corrupted value for key {key}: {error}"))
}
