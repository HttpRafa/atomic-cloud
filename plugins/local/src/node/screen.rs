use crate::generated::exports::plugin::system::{bridge::ErrorMessage, screen::GuestGenericScreen};

pub struct Screen {}

impl GuestGenericScreen for Screen {
    fn pull(&self) -> Result<Vec<String>, ErrorMessage> {
        todo!()
    }
}
