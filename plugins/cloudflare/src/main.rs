#![no_main]
#![warn(clippy::all, clippy::pedantic)]

use dummy::{node::Node, screen::Screen};
use generated::{
    export,
    exports::plugin::system::{
        bridge,
        event::{self},
        screen,
    },
};
use listener::Listener;
use plugin::Cloudflare;

mod dummy;
mod listener;
mod log;
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
    type Plugin = Cloudflare;
    type Node = Node;
}

impl screen::Guest for Export {
    type Screen = Screen;
}

impl event::Guest for Export {
    type Listener = Listener;
}

export!(Export with_types_in generated);
