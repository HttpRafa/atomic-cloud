use crate::generated::exports::plugin::system::{bridge::ErrorMessage, screen::GuestScreen};

pub struct Screen;

impl GuestScreen for Screen {
    fn pull(&self) -> Result<Vec<String>, ErrorMessage> {
        Ok(vec![])
    }

    fn write(&self, _: Vec<u8>) -> Result<(), ErrorMessage> {
        Ok(())
    }
}
