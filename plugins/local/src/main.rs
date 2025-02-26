#![no_main]

use generated::{
    export,
    exports::plugin::system::{bridge, screen},
};
use node::{screen::Screen, Node};
use plugin::Local;

mod log;
mod node;
mod plugin;
mod storage;
mod template;

#[allow(clippy::all)]
pub mod generated {
    use wit_bindgen::generate;

    generate!({
        world: "plugin",
        path: "../../protocol/wit/",
    });
}

struct Export;

impl bridge::Guest for Export {
    type Plugin = Local;
    type Node = Node;
}

impl screen::Guest for Export {
    type Screen = Screen;
}

export!(Export with_types_in generated);
