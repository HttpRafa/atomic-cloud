use super::{
    generated::cloudlet::driver::{self, platform::Os},
    WasmDriverState,
};

impl driver::platform::Host for WasmDriverState {
    fn get_os(&mut self) -> Os {
        if cfg!(target_os = "windows") {
            Os::Windows
        } else {
            Os::Unix
        }
    }
}
