use std::fs;

use common::config::LoadFromTomlFile;
use serde::{Deserialize, Serialize};
use stored::StoredTemplate;

use crate::{
    cloudlet::driver::{
        platform::{get_os, Os},
        process::{read_line, spawn_process, try_wait, Directory, Reference, StdReader},
    },
    info,
    storage::Storage,
    warn,
};

pub struct Templates {
    /* Templates */
    pub templates: Vec<Template>,
}

impl Templates {
    pub fn new() -> Self {
        Self { templates: vec![] }
    }

    pub fn load_all(&mut self) {
        info!("Loading templates...");

        let templates_directory = Storage::get_templates_folder();
        if !templates_directory.exists() {
            if let Err(error) = fs::create_dir_all(&templates_directory) {
                warn!(
                    "<red>Failed</> to create templates directory: <red>{}</>",
                    error
                );
                return;
            }
        }

        let entries = match fs::read_dir(&templates_directory) {
            Ok(entries) => entries,
            Err(error) => {
                warn!(
                    "<red>Failed</> to read templates directory: <red>{}</>",
                    &error
                );
                return;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    warn!(
                        "<red>Failed</> to read entry in templates directory: <red>{}</>",
                        &error
                    );
                    continue;
                }
            };

            let path = entry.path();
            if path.is_file() {
                continue;
            }

            let name: String = match path.file_stem() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };

            let data_file = Storage::get_template_data_file(&name);
            let template = match StoredTemplate::load_from_file(&data_file) {
                Ok(profile) => profile,
                Err(error) => {
                    warn!(
                        "<red>Failed</> to load template from file {}: <red>{}</>",
                        path.display(),
                        &error
                    );
                    continue;
                }
            };

            let template = Template::from(&name, &template);
            info!(
                "Loaded template <blue>{} {}</> by <blue>{}</>",
                name,
                template.version,
                template.authors.join(", ")
            );

            self.add_template(template);
        }

        info!("Loaded <blue>{} template(s)</>", self.templates.len());
    }

    pub fn prepare_all(&mut self) {
        info!("Running preperation scripts...");

        for template in &self.templates {
            info!(
                "Running preperation script for template <blue>{}</>",
                template.name
            );

            let prepare = match get_os() {
                Os::Unix => &template.prepare.unix,
                Os::Windows => &template.prepare.windows,
            };

            let prepare = match prepare {
                Some(script) => script,
                None => {
                    warn!(
                        "The template <blue>{}</> does not seem to support the current platform",
                        template.name
                    );
                    continue;
                }
            };

            match spawn_process(
                &prepare.command,
                &prepare.args,
                &Directory {
                    path: Storage::get_template_folder_outside(&template.name)
                        .to_string_lossy()
                        .to_string(),
                    reference: Reference::Data,
                },
            ) {
                Ok(pid) => {
                    while try_wait(pid).ok().flatten().is_none() {
                        match read_line(pid, StdReader::Stdout) {
                            Ok(read) => if read.0 > 0 {
                                let line = read.1.trim();
                                info!("<blue>[PREPARE]</> {}", line);
                            }
                            Err(error) => {
                                warn!(
                                    "Failed to read stdout of process <blue>{}</>: <red>{}</>",
                                    pid, error
                                );
                                break;
                            }
                        }
                    }
                }
                Err(error) => {
                    warn!(
                        "Failed to prepare template <blue>{}</>: <red>{}</>",
                        template.name, error
                    );
                }
            }
        }
    }

    fn add_template(&mut self, template: Template) {
        self.templates.push(template);
    }
}

#[derive(Clone)]
pub struct Template {
    /* Template */
    pub name: String,
    //pub description: String,
    pub version: String,
    pub authors: Vec<String>,

    /* Scripts */
    pub prepare: PlatformScript,
    //pub startup: PlatformScript,
}

impl Template {
    fn from(name: &str, stored: &StoredTemplate) -> Self {
        Self {
            name: name.to_string(),
            //description: stored.description.clone(),
            version: stored.version.clone(),
            authors: stored.authors.clone(),
            prepare: stored.prepare.clone(),
            //startup: stored.startup.clone(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlatformScript {
    pub unix: Option<Script>,
    pub windows: Option<Script>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Script {
    pub command: String,
    pub args: Vec<String>,
}

mod stored {

    use common::config::{LoadFromTomlFile, SaveToTomlFile};
    use serde::{Deserialize, Serialize};

    use super::PlatformScript;

    #[derive(Serialize, Deserialize)]
    pub struct StoredTemplate {
        /* Template */
        pub description: String,
        pub version: String,
        pub authors: Vec<String>,

        /* Scripts */
        pub prepare: PlatformScript,
        pub startup: PlatformScript,
    }

    impl LoadFromTomlFile for StoredTemplate {}
    impl SaveToTomlFile for StoredTemplate {}
}
