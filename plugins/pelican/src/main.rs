#![no_main]

use generated::{
    export,
    exports::plugin::system::{bridge, screen},
};
use node::{screen::Screen, Node};
use plugin::Pelican;

mod log;
mod node;
mod plugin;
mod storage;

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
    type Plugin = Pelican;
    type Node = Node;
}

impl screen::Guest for Export {
    type Screen = Screen;
}

export!(Export with_types_in generated);
