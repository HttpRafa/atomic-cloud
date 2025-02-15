use std::path::PathBuf;

use anyhow::{anyhow, Result};
use getset::Getters;
use serde::Deserialize;

use crate::{
    generated::plugin::system::{
        platform::{get_os, Os},
        process::{Process, ProcessBuilder},
    },
    storage::Storage,
};

pub mod manager;

pub struct Template {
    /* Template */
    name: String,
    version: String,
    authors: Vec<String>,

    /* Files */
    exclusions: Vec<PathBuf>,

    /* Environment */
    environment: Vec<(String, String)>,

    /* Commands */
    shutdown: Option<String>,

    /* Scripts */
    prepare: PlatformScript,
    startup: PlatformScript,
}

impl Template {
    pub fn spawn_prepare(&self) -> Result<Process> {
        let prepare = match match get_os() {
            Os::Unix => &self.prepare.unix,
            Os::Windows => &self.prepare.windows,
        } {
            Some(prepare) => prepare,
            None => return Err(anyhow!("No prepare script found for current platform. This indicates that the template is not compatible with the current platform.")),
        };

        let builder = ProcessBuilder::new(&prepare.command);
        builder.args(&prepare.args);
        builder.environment(&self.environment);
        builder.directory(&Storage::create_template_directory(&self.name));
        builder.spawn().map_err(|error| anyhow!(error))
    }
}

#[derive(Deserialize, Getters, Clone)]
pub struct PlatformScript {
    #[getset(get = "pub")]
    unix: Option<Script>,
    #[getset(get = "pub")]
    windows: Option<Script>,
}

#[derive(Deserialize, Getters, Clone)]
pub struct Script {
    #[getset(get = "pub")]
    command: String,
    #[getset(get = "pub")]
    args: Vec<String>,
}
