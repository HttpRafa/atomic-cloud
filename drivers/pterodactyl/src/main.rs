#![no_main]

use driver::{Pterodactyl, PterodactylNodeWrapper};
use exports::node::driver::bridge::Guest;
use wit_bindgen::generate;

mod config;
mod driver;
mod log;

generate!({
    world: "driver",
    path: "../../protocol/wit/",
    additional_derives: [PartialEq, Eq],
});

struct Export;

impl Guest for Export {
    type GenericDriver = Pterodactyl;
    type GenericNode = PterodactylNodeWrapper;
}

export!(Export);
