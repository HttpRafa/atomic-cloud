use std::cell::RefCell;

use crate::generated::exports::plugin::system::{bridge::ErrorMessage, screen::GuestScreen};

pub struct Screen(pub String, pub RefCell<bool>);

impl GuestScreen for Screen {
    fn pull(&self) -> Result<Vec<String>, ErrorMessage> {
        if *self.1.borrow() {
            self.1.replace(false);
            return Ok(vec![format!(
                "Please use the pelican panel under this url({}) to interact with the screen.",
                self.0
            )]);
        }
        Ok(vec![])
    }

    fn write(&self, _: Vec<u8>) -> Result<(), ErrorMessage> {
        Ok(())
    }
}
