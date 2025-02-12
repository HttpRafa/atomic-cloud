#![no_main]

use plugin::{Pterodactyl, PterodactylCloudletWrapper};
use exports::node::plugin::bridge::Guest;
use wit_bindgen::generate;

mod plugin;
mod log;
mod storage;

generate!({
    world: "plugin",
    path: "../../protocol/wit/",
    additional_derives: [PartialEq, Eq],
});

struct Export;

impl Guest for Export {
    type GenericDriver = Pterodactyl;
    type GenericCloudlet = PterodactylCloudletWrapper;
}

export!(Export);
