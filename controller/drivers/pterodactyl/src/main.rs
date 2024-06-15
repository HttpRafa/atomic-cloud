#![no_main]

use driver::{Pterodactyl, PterodactylNodeWrapper};
use exports::node::driver::bridge::Guest;
use wit_bindgen::generate;

mod driver;
mod config;
mod log;

generate!({
    world: "driver",
    path: "../../structure/wit/"
});

struct Export;

impl Guest for Export {
    type GenericDriver = Pterodactyl;
    type GenericNode = PterodactylNodeWrapper;
}

export!(Export);