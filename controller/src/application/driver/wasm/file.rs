use std::fs;

use super::{
    generated::cloudlet::driver::{
        self,
        types::{Directory, ErrorMessage},
    },
    WasmDriverState,
};

impl driver::file::Host for WasmDriverState {
    fn remove_dir_all(&mut self, directory: Directory) -> Result<(), ErrorMessage> {
        let driver = self.handle.upgrade().ok_or("Failed to upgrade handle")?;
        let path = self.get_directory(&driver.name, &directory)?;
        fs::remove_dir_all(path).map_err(|error| format!("Failed to remove directory: {}", error))
    }
}
