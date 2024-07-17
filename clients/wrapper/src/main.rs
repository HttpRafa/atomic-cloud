use log::{info, LevelFilter};
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};
use wrapper::Wrapper;

mod wrapper;

fn main() {
    init_logging();

    info!("Starting wrapper...");
    let mut wrapper = Wrapper::new();
    wrapper.start();
}

#[cfg(debug_assertions)]
fn init_logging() {
    TermLogger::init(
        LevelFilter::Debug,
        ConfigBuilder::new()
            .set_location_level(LevelFilter::Error)
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .expect("Failed to init logging crate");
}

#[cfg(not(debug_assertions))]
fn init_logging() {
    TermLogger::init(
        LevelFilter::Info,
        ConfigBuilder::new().build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .expect("Failed to init logging crate");
}
