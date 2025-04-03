use crate::generated::{
    exports::plugin::system::screen::GuestScreen, plugin::system::types::ErrorMessage,
};

pub struct Screen();

impl GuestScreen for Screen {
    fn pull(&self) -> Result<Vec<String>, ErrorMessage> {
        unimplemented!()
    }

    fn write(&self, _: Vec<u8>) -> Result<(), ErrorMessage> {
        unimplemented!()
    }
}
