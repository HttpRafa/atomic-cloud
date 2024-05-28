use wit_bindgen::generate;

generate!({
    world: "driver",
    path: "../../structure/wit/"
});

struct Pelican {}

impl Guest for Pelican {
    fn init() {
        info("Hello, world");
    }
}

export!(Pelican);