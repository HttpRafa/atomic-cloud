#![no_main]

use std::fs;

use exports::node::driver::bridge::{Guest, Information, Node};
use node::driver::log::{lerror, linfo};
use wit_bindgen::generate;

generate!({
    world: "driver",
    path: "../../structure/wit/"
});

const AUTHORS: [&str; 1] = ["HttpRafa"];
const VERSION: &str = "0.1.0";

struct Pelican {}

impl Guest for Pelican {
    fn init() -> Information {
        fs::write("config.toml", "Test Config").expect("Failed to write config file");
        fs::write("./data/images.txt", "image1,image2").expect("Failed to write test file");
        Information {
            authors: AUTHORS.map(|author|author.to_string()).to_vec(),
            version: VERSION.to_string(),
        }
    }
    fn init_node(_node: Node) -> bool {
        true
    }
}

export!(Pelican);