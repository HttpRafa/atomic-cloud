//! Persistent key-value store backing the wasmite `kv` interface.
//!
//! Every wasmite plugin instance lives for a single call, so this store is
//! the only place a plugin can keep state between calls. All operations on
//! the in-memory map are atomic with respect to each other, and every
//! mutation is written to disk (via a temporary file and a rename) before
//! the call returns, so a crash never leaves a half-written file behind.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::{Context as _, Result, bail};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use tokio::{fs, io::AsyncWriteExt, sync::Mutex as AsyncMutex};

/// Upper bound for a single key. Keys are short identifiers in practice.
pub const MAX_KEY_LENGTH: usize = 1024;
/// Upper bound for a single value, to catch runaway plugins early.
pub const MAX_VALUE_LENGTH: usize = 4 * 1024 * 1024;

#[derive(Serialize, Deserialize, Default)]
struct Snapshot {
    #[serde(default)]
    entries: BTreeMap<String, String>,
}

pub struct KvStore {
    path: PathBuf,
    entries: Mutex<BTreeMap<String, Vec<u8>>>,
    /// Serializes snapshot writes so an older snapshot can never overwrite
    /// a newer one.
    persist: AsyncMutex<()>,
}

impl KvStore {
    /// Opens the store at `path`, loading existing entries if the file exists.
    pub async fn open(path: PathBuf) -> Result<Self> {
        let entries = if path.exists() {
            let data = fs::read_to_string(&path)
                .await
                .with_context(|| format!("Failed to read kv store {}", path.display()))?;
            let snapshot: Snapshot = toml::from_str(&data)
                .with_context(|| format!("Failed to parse kv store {}", path.display()))?;
            snapshot
                .entries
                .into_iter()
                .map(|(key, value)| {
                    STANDARD
                        .decode(value)
                        .map(|value| (key.clone(), value))
                        .with_context(|| format!("Invalid value for key {key} in kv store"))
                })
                .collect::<Result<_>>()?
        } else {
            BTreeMap::new()
        };

        Ok(Self {
            path,
            entries: Mutex::new(entries),
            persist: AsyncMutex::new(()),
        })
    }

    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.lock().get(key).cloned()
    }

    pub async fn set(&self, key: &str, value: Vec<u8>) -> Result<()> {
        validate(key, Some(&value))?;
        let changed = {
            let mut entries = self.lock();
            if entries.get(key) == Some(&value) {
                false
            } else {
                entries.insert(key.to_string(), value);
                true
            }
        };
        if changed {
            self.persist().await?;
        }
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        let changed = self.lock().remove(key).is_some();
        if changed {
            self.persist().await?;
        }
        Ok(())
    }

    /// Replaces the value of `key` with `new` if the current value equals
    /// `expected` (`None` meaning "absent"). Returns whether it swapped.
    pub async fn compare_and_swap(
        &self,
        key: &str,
        expected: Option<&[u8]>,
        new: Option<Vec<u8>>,
    ) -> Result<bool> {
        validate(key, new.as_deref())?;
        let changed = {
            let mut entries = self.lock();
            if entries.get(key).map(Vec::as_slice) != expected {
                return Ok(false);
            }
            let changed = entries.get(key) != new.as_ref();
            match new {
                Some(value) => entries.insert(key.to_string(), value),
                None => entries.remove(key),
            };
            changed
        };
        if changed {
            self.persist().await?;
        }
        Ok(true)
    }

    /// All keys starting with `prefix`, in sorted order.
    pub fn keys(&self, prefix: &str) -> Vec<String> {
        self.lock()
            .range(prefix.to_string()..)
            .take_while(|(key, _)| key.starts_with(prefix))
            .map(|(key, _)| key.clone())
            .collect()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, BTreeMap<String, Vec<u8>>> {
        // A poisoned lock only means another thread panicked mid-operation;
        // every operation leaves the map consistent, so keep using it.
        self.entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    async fn persist(&self) -> Result<()> {
        let _guard = self.persist.lock().await;
        // Take the snapshot while holding the persist lock, so that the last
        // writer always writes the latest state.
        let snapshot = Snapshot {
            entries: self
                .lock()
                .iter()
                .map(|(key, value)| (key.clone(), STANDARD.encode(value)))
                .collect(),
        };
        write_atomically(&self.path, toml::to_string(&snapshot)?.as_bytes()).await
    }
}

fn validate(key: &str, value: Option<&[u8]>) -> Result<()> {
    if key.is_empty() {
        bail!("Key must not be empty");
    }
    if key.len() > MAX_KEY_LENGTH {
        bail!("Key exceeds {MAX_KEY_LENGTH} bytes");
    }
    if value.is_some_and(|value| value.len() > MAX_VALUE_LENGTH) {
        bail!("Value exceeds {MAX_VALUE_LENGTH} bytes");
    }
    Ok(())
}

/// Writes `data` to `path` by writing a sibling temporary file, syncing it
/// and renaming it over the target, so readers see either the old or the
/// new content but never a partial file.
pub async fn write_atomically(path: &Path, data: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".tmp");
    let temporary = PathBuf::from(temporary);

    let mut file = fs::File::create(&temporary)
        .await
        .with_context(|| format!("Failed to create {}", temporary.display()))?;
    file.write_all(data).await?;
    file.sync_all().await?;
    drop(file);
    fs::rename(&temporary, path)
        .await
        .with_context(|| format!("Failed to replace {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn store() -> (tempfile::TempDir, KvStore) {
        let directory = tempfile::tempdir().unwrap();
        let store = KvStore::open(directory.path().join("kv.toml"))
            .await
            .unwrap();
        (directory, store)
    }

    #[tokio::test]
    async fn set_get_delete() {
        let (_directory, store) = store().await;
        assert_eq!(store.get("a"), None);
        store.set("a", b"1".to_vec()).await.unwrap();
        assert_eq!(store.get("a"), Some(b"1".to_vec()));
        store.delete("a").await.unwrap();
        assert_eq!(store.get("a"), None);
        // Deleting a missing key is fine
        store.delete("a").await.unwrap();
    }

    #[tokio::test]
    async fn compare_and_swap_semantics() {
        let (_directory, store) = store().await;

        // Insert only if absent
        assert!(
            store
                .compare_and_swap("k", None, Some(b"v1".to_vec()))
                .await
                .unwrap()
        );
        assert!(
            !store
                .compare_and_swap("k", None, Some(b"v2".to_vec()))
                .await
                .unwrap()
        );

        // Swap with the wrong expectation fails and leaves the value alone
        assert!(
            !store
                .compare_and_swap("k", Some(b"nope"), Some(b"v2".to_vec()))
                .await
                .unwrap()
        );
        assert_eq!(store.get("k"), Some(b"v1".to_vec()));

        // Swap with the right expectation succeeds
        assert!(
            store
                .compare_and_swap("k", Some(b"v1"), Some(b"v2".to_vec()))
                .await
                .unwrap()
        );
        assert_eq!(store.get("k"), Some(b"v2".to_vec()));

        // `new: None` deletes
        assert!(
            store
                .compare_and_swap("k", Some(b"v2"), None)
                .await
                .unwrap()
        );
        assert_eq!(store.get("k"), None);
    }

    #[tokio::test]
    async fn keys_by_prefix() {
        let (_directory, store) = store().await;
        for key in ["server/a", "server/b", "serverx", "template/a", "s"] {
            store.set(key, vec![1]).await.unwrap();
        }
        assert_eq!(store.keys("server/"), vec!["server/a", "server/b"]);
        assert_eq!(store.keys(""), {
            let mut all = vec!["s", "server/a", "server/b", "serverx", "template/a"];
            all.sort_unstable();
            all
        });
        assert!(store.keys("zzz").is_empty());
    }

    #[tokio::test]
    async fn survives_reopen() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("nested").join("kv.toml");
        {
            let store = KvStore::open(path.clone()).await.unwrap();
            store.set("binary", vec![0, 159, 255]).await.unwrap();
            store.set("text", b"hello".to_vec()).await.unwrap();
            store.delete("text").await.unwrap();
        }
        let store = KvStore::open(path.clone()).await.unwrap();
        assert_eq!(store.get("binary"), Some(vec![0, 159, 255]));
        assert_eq!(store.get("text"), None);
        // No temporary file is left behind
        assert!(!directory.path().join("nested").join("kv.toml.tmp").exists());
    }

    #[tokio::test]
    async fn concurrent_increments_do_not_lose_updates() {
        let (_directory, store) = store().await;
        let store = std::sync::Arc::new(store);
        let mut tasks = vec![];
        for _ in 0..16 {
            let store = store.clone();
            tasks.push(tokio::spawn(async move {
                for _ in 0..10 {
                    loop {
                        let current = store.get("counter");
                        let value = current
                            .as_deref()
                            .map_or(0u32, |bytes| u32::from_le_bytes(bytes.try_into().unwrap()));
                        let next = (value + 1).to_le_bytes().to_vec();
                        if store
                            .compare_and_swap("counter", current.as_deref(), Some(next))
                            .await
                            .unwrap()
                        {
                            break;
                        }
                    }
                }
            }));
        }
        for task in tasks {
            task.await.unwrap();
        }
        let value = u32::from_le_bytes(store.get("counter").unwrap().try_into().unwrap());
        assert_eq!(value, 160);
    }

    #[tokio::test]
    async fn rejects_invalid_input() {
        let (_directory, store) = store().await;
        assert!(store.set("", vec![]).await.is_err());
        assert!(
            store
                .set(&"k".repeat(MAX_KEY_LENGTH + 1), vec![])
                .await
                .is_err()
        );
        assert!(
            store
                .set("big", vec![0; MAX_VALUE_LENGTH + 1])
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn corrupted_file_is_an_error() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("kv.toml");
        fs::write(&path, "entries = { a = \"not base64!\" }")
            .await
            .unwrap();
        assert!(KvStore::open(path).await.is_err());
    }
}
