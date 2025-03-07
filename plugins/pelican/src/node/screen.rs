use crate::generated::exports::plugin::system::{bridge::ErrorMessage, screen::GuestScreen};

pub struct Screen;

impl GuestScreen for Screen {
    fn pull(&self) -> Result<Vec<String>, ErrorMessage> {
        unimplemented!("Screen::pull")
    }

    fn write(&self, _: Vec<u8>) -> Result<(), ErrorMessage> {
        unimplemented!("Screen::write")
    }
}
