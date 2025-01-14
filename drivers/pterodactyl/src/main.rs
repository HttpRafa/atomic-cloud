#![no_main]

use driver::{Pterodactyl, PterodactylCloudletWrapper};
use exports::cloudlet::driver::bridge::Guest;
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
    type GenericCloudlet = PterodactylCloudletWrapper;
}

export!(Export);
