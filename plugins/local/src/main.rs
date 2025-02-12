#![no_main]

use plugin::{Local, LocalCloudletWrapper};
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
    type GenericDriver = Local;
    type GenericCloudlet = LocalCloudletWrapper;
}

export!(Export);
