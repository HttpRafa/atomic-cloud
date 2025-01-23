use std::fs;

use super::{
    generated::cloudlet::driver::{self, types::Directory},
    WasmDriverState,
};

impl driver::file::Host for WasmDriverState {
    fn remove_dir_all(&mut self, directory: Directory) -> Result<bool, String> {
        let driver = self.handle.upgrade().ok_or("Failed to upgrade handle")?;
        let path = self.get_directory(&driver.name, &directory)?;
        fs::remove_dir_all(path)
            .map_err(|e| format!("Failed to remove directory: {}", e))
            .map(|_| true)
    }
}
