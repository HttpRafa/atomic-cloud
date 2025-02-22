use std::rc::Rc;

use crate::generated::{
    exports::plugin::system::{bridge::ErrorMessage, screen::GuestGenericScreen},
    plugin::system::process::Process,
};

pub struct Screen(pub Rc<Process>);

impl GuestGenericScreen for Screen {
    fn pull(&self) -> Result<Vec<String>, ErrorMessage> {
        Ok(self.0.read_lines())
    }

    fn write(&self, data: Vec<u8>) -> Result<(), ErrorMessage> {
        self.0.write_all(&data)?;
        self.0.flush()
    }
}
