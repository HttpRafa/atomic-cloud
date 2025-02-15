#![no_main]

use generated::{
    export,
    exports::plugin::system::{bridge, screen},
};
use node::{screen::Screen, Node};
use plugin::Local;

mod log;
mod template;
mod node;
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

impl bridge::Guest for Export {
    type GenericPlugin = Local;
    type GenericNode = Node;
}

impl screen::Guest for Export {
    type GenericScreen = Screen;
}

export!(Export with_types_in generated);
