use std::path::Path;

use anyhow::Result;
use common::name::TimedName;

use crate::driver::template::Template;

pub struct LocalUnit {
    pub _pid: u32,
    pub _name: TimedName,
}

impl LocalUnit {
    pub fn start(name: &TimedName, folder: &Path, template: &Template) -> Result<Self> {
        Ok(Self {
            _pid: template.run_startup(folder)?,
            _name: name.clone(),
        })
    }
}
