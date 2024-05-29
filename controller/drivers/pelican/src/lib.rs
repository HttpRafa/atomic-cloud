use exports::node::driver::bridge::{Guest, Information};
use node::driver::log::linfo;
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
        linfo("message");
        Information {
            authors: AUTHORS.map(|author|author.to_string()).to_vec(),
            version: VERSION.to_string(),
        }
    }
}

export!(Pelican);