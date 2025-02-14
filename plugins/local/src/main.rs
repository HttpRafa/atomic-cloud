#![no_main]

use generated::{
    export,
    exports::plugin::system::{bridge, screen},
};

mod log;
mod plugin;
mod storage;

#[allow(clippy::all)]
pub mod generated {
    use wit_bindgen::generate;

    generate!({
        world: "plugin",
        path: "../../protocol/wit/",
        async: true,
    });
}

struct Export;

impl bridge::Guest for Export {}

impl screen::Guest for Export {}

export!(Export with_types_in generated);
