#![no_main]
#![feature(let_chains)]
#![warn(clippy::all, clippy::pedantic)]

use generated::{
    export,
    exports::plugin::system::{bridge, event, screen},
};
use listener::Listener;
use node::{screen::Screen, Node};
use plugin::Pelican;

mod listener;
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

impl event::Guest for Export {
    type Listener = Listener;
}

export!(Export with_types_in generated);
