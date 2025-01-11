use simplelog::{debug, error, info, warn};

use super::{
    generated::cloudlet::driver::{self, log::Level},
    WasmDriverState,
};

impl driver::log::Host for WasmDriverState {
    fn log_string(&mut self, level: Level, message: String) {
        match level {
            Level::Info => info!(
                "<blue>[{}]</> {}",
                &self.handle.upgrade().unwrap().name.to_uppercase(),
                message
            ),
            Level::Warn => warn!(
                "<blue>[{}]</> {}",
                &self.handle.upgrade().unwrap().name.to_uppercase(),
                message
            ),
            Level::Error => error!(
                "<blue>[{}] {}",
                &self.handle.upgrade().unwrap().name.to_uppercase(),
                message
            ),
            Level::Debug => debug!(
                "[{}] {}",
                &self.handle.upgrade().unwrap().name.to_uppercase(),
                message
            ),
        }
    }
}
