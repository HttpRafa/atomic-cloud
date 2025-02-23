use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use getset::Getters;
use serde::Deserialize;
use walkdir::WalkDir;

use crate::{
    generated::plugin::system::{
        file::Directory,
        platform::{get_os, Os},
        process::{Process, ProcessBuilder},
    },
    storage::Storage,
};

pub mod manager;

#[derive(Getters)]
pub struct Template {
    /* Template */
    #[getset(get = "pub")]
    name: String,
    #[allow(unused)]
    version: String,
    #[allow(unused)]
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

    pub fn spawn(
        &self,
        data: Vec<(String, String)>,
        directory: &Directory,
    ) -> Result<(Process, ProcessBuilder)> {
        let startup = match match get_os() {
            Os::Unix => &self.startup.unix,
            Os::Windows => &self.startup.windows,
        } {
            Some(startup) => startup,
            None => return Err(anyhow!("No startup script found for current platform. This indicates that the template is not compatible with the current platform.")),
        };

        let mut environment = self.environment.clone();
        environment.extend(data);

        let builder = ProcessBuilder::new(&startup.command);
        builder.args(&startup.args);
        builder.environment(&environment);
        builder.directory(directory);
        Ok((builder.spawn().map_err(|error| anyhow!(error))?, builder))
    }

    pub fn write_shutdown(&self, process: &Process) -> Result<()> {
        match &self.shutdown {
            Some(command) => process
                .write_all(format!("{}\n", command).as_bytes())
                .map_err(|error| {
                    anyhow!("Failed to send shutdown command to process: {}", error)
                })?,
            None => process
                .write_all("^C\n".as_bytes())
                .map_err(|error| anyhow!("Failed to send Ctl+C command to process: {}", error))?,
        }

        process
            .flush()
            .map_err(|error| anyhow!("Failed to flush processes stdin: {}", error))?;
        Ok(())
    }

    pub fn copy_self(&self, to: &Path) -> Result<()> {
        let from = Storage::template_directory(false, &self.name);
        fs::create_dir_all(to)?;

        for entry in WalkDir::new(&from) {
            let entry = entry?;
            let path = entry.path();

            // Skip directories as they'll be created automatically with files
            if path.is_dir() {
                continue;
            }

            // Calculate the relative path
            let relative_path = path.strip_prefix(&from).unwrap();

            // Check if the relative path is in the exclusions list
            if self
                .exclusions
                .iter()
                .any(|exclusion| exclusion == relative_path)
            {
                continue;
            }

            // Construct the destination path
            let destination = to.join(relative_path);

            // Ensure the destination directory exists
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }

            // Copy the file
            fs::copy(path, &destination)?;
        }

        Ok(())
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
