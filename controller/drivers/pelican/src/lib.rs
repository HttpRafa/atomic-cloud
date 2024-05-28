use wit_bindgen::generate;

generate!({
    world: "driver",
});

struct Pelican {}

impl Guest for Pelican {
    fn run() {
        print("Hello, world");
    }
}

export!(Pelican);