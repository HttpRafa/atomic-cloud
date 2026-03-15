use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;
use wasmtime::Engine;

use crate::{application::{Shared, plugin::{Features, runtime::wasm::config::Permissions}}, task::manager::TaskSender};

pub mod plugin;

#[allow(clippy::all)]
pub mod generated {
    use wasmtime::component::bindgen;

    bindgen!({
        world: "plugin",
        path: "../protocol/wasmite/",
        imports: { default: async | trappable },
        exports: { default: async },
    });
}

pub type StoreMap<D> = RwLock<HashMap<String, D>>;

pub struct Context {
    /* Global */
    tasks: TaskSender,
    shared: Arc<Shared>,

    /* Plugin */
    name: String,
    permissions: Permissions,

    /* State */
    store: (StoreMap<String>, StoreMap<i32>, StoreMap<f32>, StoreMap<Vec<u8>>),
}

pub struct Plugin {
    /* Features */
    features: Features,

    /* Wasmtime */
    engine: Engine,

    /* Context */
    context: Arc<Context>,
}