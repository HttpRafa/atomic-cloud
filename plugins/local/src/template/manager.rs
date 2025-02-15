use std::{collections::HashMap, fs};

use anyhow::{anyhow, Result};
use stored::StoredTemplate;

use crate::{error, info, storage::Storage};

use super::Template;

#[derive(Default)]
pub struct TemplateManager {
    templates: HashMap<String, Template>,
}

impl TemplateManager {
    pub fn init(&mut self) -> Result<()> {
        info!("Loading templates...");

        let directory = Storage::templates_directory(false);
        if !directory.exists() {
            fs::create_dir_all(&directory)?;
        }

        for (_, _, name, value) in Storage::for_each_file_in_directory_toml::<StoredTemplate>(
            Storage::template_data_file_name(),
            &directory,
            "Failed to read template from file",
        )? {
            info!("Loaded template {}", name);
            self.templates
                .insert(name.clone(), Template::new(&name, &value));
        }

        info!("Loaded {} template(s)", self.templates.len());
        Ok(())
    }

    pub fn run_prepare(&self) -> Result<()> {
        let mut processes = Vec::with_capacity(self.templates.len());
        for template in self.templates.values() {
            match template.spawn_prepare() {
                Ok(process) => processes.push((template, process)),
                Err(error) => error!(
                    "Failed to run prepare script for template {}: {}",
                    template.name, error
                ),
            }
        }

        loop {
            let mut running = false;

            for (template, process) in &processes {
                if process
                    .try_wait()
                    .map_err(|error| anyhow!(error))?
                    .is_none()
                {
                    running = true; // At least one process is still running
                    for line in process.read_lines() {
                        info!("[{}] {}", template.name, line.trim());
                    }
                }
            }

            if !running {
                break;
            }
        }

        Ok(())
    }
}

impl Template {
    pub fn new(name: &str, template: &StoredTemplate) -> Self {
        Self {
            name: name.to_owned(),
            version: template.version().to_string(),
            authors: template.authors().clone(),
            exclusions: template.exclusions().clone(),
            environment: template
                .environment()
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            shutdown: template.shutdown().clone(),
            prepare: template.prepare().clone(),
            startup: template.startup().clone(),
        }
    }
}

pub(super) mod stored {
    use std::{collections::HashMap, path::PathBuf};

    use common::file::SyncLoadFromTomlFile;
    use getset::Getters;
    use serde::Deserialize;

    use crate::template::PlatformScript;

    #[derive(Deserialize, Getters)]
    pub struct StoredTemplate {
        /* Template */
        #[getset(get = "pub")]
        description: String,
        #[getset(get = "pub")]
        version: String,
        #[getset(get = "pub")]
        authors: Vec<String>,

        /* Files */
        #[getset(get = "pub")]
        exclusions: Vec<PathBuf>,

        /* Environment */
        #[getset(get = "pub")]
        environment: HashMap<String, String>,

        /* Commands */
        #[getset(get = "pub")]
        shutdown: Option<String>,

        /* Scripts */
        #[getset(get = "pub")]
        prepare: PlatformScript,
        #[getset(get = "pub")]
        startup: PlatformScript,
    }

    impl SyncLoadFromTomlFile for StoredTemplate {}
}
