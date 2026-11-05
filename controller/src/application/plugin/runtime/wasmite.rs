//! The wasmite plugin runtime.
//!
//! Unlike the `wasm` runtime, which keeps one long-lived instance per
//! plugin, wasmite creates a fresh instance for every call and drops it
//! afterwards. Everything that has to outlive a call lives in [`Context`]
//! on the host: the kv store, child processes and stop guards.
//!
//! Benefits of this model:
//! - a trap, a leak or a stuck call only affects that one call, and a call
//!   can be cancelled on timeout because its store is thrown away anyway
//! - calls into the same plugin run concurrently instead of queueing on a
//!   single store lock
//! - plugin state is persistent, because it lives in the host kv store

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::{Result, anyhow};
use tokio::time::timeout;
use wasmtime::{Engine, Store, StoreLimits, StoreLimitsBuilder};
use wasmtime_wasi::{FsPerms, ResourceTable, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

use crate::application::{plugin::Features, server::guard::Guard};

use super::config::Permissions;

mod host;
pub mod init;
pub mod kv;
pub mod node;
pub mod plugin;
pub mod process;
pub mod screen;

#[allow(clippy::all, clippy::pedantic)]
pub mod generated {
    use wasmtime::component::bindgen;

    bindgen!({
        world: "plugin",
        path: "../protocol/wasmite/",
        imports: { default: async | trappable },
        exports: { default: async },
    });
}

/// Longest time a single call into a plugin may take. The instance is
/// discarded afterwards anyway, so cancelling it on timeout is safe.
const CALL_TIMEOUT: Duration = Duration::from_secs(30);
/// Linear memory limit for a single plugin instance.
const MEMORY_LIMIT: usize = 256 * 1024 * 1024;

/// Host-side state of one plugin that outlives individual calls.
pub struct Context {
    pub name: String,
    pub identifier: String,
    pub permissions: Permissions,
    pub certificate: Option<String>,

    /* Filesystem */
    pub data_directory: PathBuf,
    pub config_directory: PathBuf,
    pub mounts: Vec<(String, String)>,

    /* State */
    pub kv: kv::KvStore,
    pub processes: process::ProcessRegistry,
    /// Guards of servers that are being stopped, keyed by server uuid. The
    /// controller considers a stop complete once its guard is dropped.
    pub guards: Mutex<HashMap<String, Guard>>,
}

impl Context {
    pub fn insert_guard(&self, server: String, guard: Guard) {
        self.guards
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(server, guard);
    }

    pub fn release_guard(&self, server: &str) {
        self.guards
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(server);
    }

    pub fn release_all_guards(&self) {
        self.guards
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }
}

/// State of a single call. Created right before the call, dropped after it.
pub struct State {
    context: Arc<Context>,
    wasi: WasiCtx,
    table: ResourceTable,
    limits: StoreLimits,
}

impl WasiView for State {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

/// Everything needed to create instances of one plugin.
pub struct Runtime {
    engine: Engine,
    pre: generated::PluginPre<State>,
    context: Arc<Context>,
}

impl Runtime {
    pub fn new(engine: Engine, pre: generated::PluginPre<State>, context: Arc<Context>) -> Self {
        Self {
            engine,
            pre,
            context,
        }
    }

    pub fn context(&self) -> &Arc<Context> {
        &self.context
    }

    /// Runs `call` on a fresh instance of the plugin.
    pub async fn call<T, F>(&self, call: F) -> Result<T>
    where
        F: AsyncFnOnce(&generated::Plugin, &mut Store<State>) -> wasmtime::Result<T>,
    {
        let mut store = Store::new(&self.engine, self.new_state()?);
        store.limiter(|state| &mut state.limits);
        // Yield back to tokio regularly so a busy plugin cannot starve the
        // executor thread it runs on.
        store.epoch_deadline_async_yield_and_update(2);

        let work = async {
            let bindings = self.pre.instantiate_async(&mut store).await?;
            call(&bindings, &mut store).await
        };
        match timeout(CALL_TIMEOUT, work).await {
            Ok(result) => result.map_err(|error| anyhow!("{error}")),
            Err(_) => Err(anyhow!(
                "Call into plugin {} timed out after {:?}",
                self.context.name,
                CALL_TIMEOUT
            )),
        }
    }

    fn new_state(&self) -> Result<State> {
        let context = &self.context;
        let mut wasi = WasiCtxBuilder::new();
        let permissions = &context.permissions;
        if permissions.contains(Permissions::INHERIT_STDIO) {
            wasi.inherit_stdio();
        }
        if permissions.contains(Permissions::INHERIT_ARGS) {
            wasi.inherit_args();
        }
        if permissions.contains(Permissions::INHERIT_ENV) {
            wasi.inherit_env();
        }
        if permissions.contains(Permissions::INHERIT_NETWORK) {
            wasi.inherit_network();
        }
        if permissions.contains(Permissions::ALLOW_IP_NAME_LOOKUP) {
            wasi.allow_ip_name_lookup(true);
        }
        for (host, guest) in &context.mounts {
            wasi.preopened_dir(host, guest, FsPerms::ReadWrite)?;
        }
        wasi.preopened_dir(&context.config_directory, "/configs", FsPerms::ReadWrite)?
            .preopened_dir(&context.data_directory, "/data", FsPerms::ReadWrite)?;

        Ok(State {
            context: context.clone(),
            wasi: wasi.build(),
            table: ResourceTable::new(),
            limits: StoreLimitsBuilder::new().memory_size(MEMORY_LIMIT).build(),
        })
    }
}

impl From<generated::atomic::plugin_system::types::Features> for Features {
    fn from(value: generated::atomic::plugin_system::types::Features) -> Self {
        let mut features = Features::empty();
        if value.contains(generated::atomic::plugin_system::types::Features::NODE) {
            features.insert(Features::NODE);
        }
        features
    }
}

/// Formats the error of a call that reports per-item errors.
pub fn scoped_errors(
    errors: &[generated::atomic::plugin_system::errors::ScopedError],
) -> anyhow::Error {
    anyhow!(
        errors
            .iter()
            .map(|error| format!("Scope: {}, Message: {}", error.scope, error.message))
            .collect::<Vec<_>>()
            .join("\n")
    )
}
