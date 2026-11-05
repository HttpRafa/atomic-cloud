//! Server templates: a directory under `/data/templates/<name>` with a
//! `template.toml` describing how to prepare and start servers from it.
//!
//! Prepare scripts (e.g. downloading the server jar) can take minutes, which
//! is far longer than one plugin call may run. So `init` only starts them
//! and records the template as preparing; `tick` forwards their output to
//! the log and marks the template ready once the script has exited.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context as _, Result, anyhow, bail};
use common::file::SyncLoadFromTomlFile;
use serde::Deserialize;
use walkdir::WalkDir;

use crate::{
    error,
    generated::atomic::plugin_system::{
        errors::{Error, ScopedError},
        platform::{Os, get_os},
        process::{self, Command, ProcessStatus},
    },
    info,
    state::{self, TEMPLATE_PREFIX, TemplateRecord, TemplateStatus},
    storage::Storage,
};

const PREPARE_PREFIX: &str = "prepare/";

#[derive(Deserialize, Clone)]
pub struct Script {
    command: String,
    args: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub struct PlatformScript {
    unix: Option<Script>,
    windows: Option<Script>,
}

impl PlatformScript {
    fn current(&self) -> Option<&Script> {
        match get_os() {
            Os::Unix => self.unix.as_ref(),
            Os::Windows => self.windows.as_ref(),
        }
    }
}

#[derive(Deserialize)]
pub struct Template {
    #[serde(skip)]
    name: String,

    #[allow(unused)]
    description: String,
    version: String,
    authors: Vec<String>,

    /* Files */
    exclusions: Vec<PathBuf>,

    /* Environment */
    environment: HashMap<String, String>,

    /* Commands */
    shutdown: Option<String>,

    /* Scripts */
    prepare: PlatformScript,
    startup: PlatformScript,
}

impl SyncLoadFromTomlFile for Template {}

impl Template {
    pub fn load(name: &str) -> Result<Self> {
        let mut template = Self::from_file(&Storage::template_data_file(name))
            .with_context(|| format!("Failed to read template {name}"))?;
        template.name = name.to_string();
        Ok(template)
    }

    /// Loads a template that is ready to start servers from.
    pub fn load_ready(name: &str) -> Result<Self> {
        match state::get::<TemplateRecord>(&state::template_key(name))? {
            Some(TemplateRecord {
                status: TemplateStatus::Ready,
                ..
            }) => Self::load(name),
            Some(TemplateRecord {
                status: TemplateStatus::Preparing,
                ..
            }) => bail!(Error::NotReady(format!(
                "Template {name} is still being prepared"
            ))),
            Some(TemplateRecord {
                status: TemplateStatus::Failed,
                message,
            }) => bail!(Error::NotReady(format!(
                "Template {name} failed to prepare: {}",
                message.unwrap_or_default()
            ))),
            None => bail!(Error::NotFound(format!("Template {name} does not exist"))),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn shutdown(&self) -> Option<&str> {
        self.shutdown.as_deref()
    }

    /// The command that starts a server from this template in `directory`
    /// (relative to the data directory), with `variables` added to the
    /// template's environment.
    pub fn startup_command(
        &self,
        directory: &str,
        variables: Vec<(String, String)>,
    ) -> Result<Command> {
        let Some(startup) = self.startup.current() else {
            bail!(Error::NotReady(format!(
                "Template {} has no startup script for this platform",
                self.name
            )));
        };
        let mut environment: Vec<(String, String)> = self
            .environment
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        environment.extend(variables);
        Ok(Command {
            program: startup.command.clone(),
            args: startup.args.clone(),
            environment,
            directory: directory.to_string(),
        })
    }

    /// Copies the template's files into `to` (a guest path).
    pub fn copy_to(&self, to: &Path) -> Result<()> {
        let from = Storage::guest(&Storage::template_directory(&self.name));
        fs::create_dir_all(to)?;

        for entry in WalkDir::new(&from) {
            let entry = entry?;
            let path = entry.path();

            // Directories are created along with the files inside them
            if path.is_dir() {
                continue;
            }

            let relative = path.strip_prefix(&from)?;
            if self
                .exclusions
                .iter()
                .any(|exclusion| exclusion == relative)
            {
                continue;
            }

            let destination = to.join(relative);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, &destination)?;
        }
        Ok(())
    }
}

/// Discovers all templates and starts their prepare scripts.
pub fn prepare_all() -> Result<()> {
    let directory = Storage::guest(&Storage::templates_directory());
    fs::create_dir_all(&directory)?;

    info!("Loading templates...");
    let mut count = 0;
    for entry in fs::read_dir(&directory)? {
        let entry = entry?;
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let template = match Template::load(&name) {
            Ok(template) => template,
            Err(error) => {
                error!("{:#}", error);
                continue;
            }
        };
        info!(
            "Loaded template {} {} by {}",
            name,
            template.version,
            template.authors.join(",")
        );
        count += 1;

        let record = match spawn_prepare(&template) {
            Ok(()) => TemplateRecord {
                status: TemplateStatus::Preparing,
                message: None,
            },
            Err(error) => {
                error!(
                    "Failed to run prepare script for template {}: {}",
                    name, error
                );
                TemplateRecord {
                    status: TemplateStatus::Failed,
                    message: Some(error.to_string()),
                }
            }
        };
        state::put(&state::template_key(&name), &record)?;
    }
    info!("Loaded {} template(s)", count);
    Ok(())
}

fn spawn_prepare(template: &Template) -> Result<()> {
    let Some(prepare) = template.prepare.current() else {
        bail!(
            "No prepare script found for current platform. This indicates that the template is not compatible with the current platform."
        );
    };
    process::spawn(
        &format!("{PREPARE_PREFIX}{}", template.name),
        &Command {
            program: prepare.command.clone(),
            args: prepare.args.clone(),
            environment: template
                .environment
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect(),
            directory: Storage::template_directory(&template.name),
        },
    )
    .map_err(|error| anyhow!(error))
}

/// Forwards prepare output to the log and finishes templates whose prepare
/// script has exited.
pub fn tick() -> Result<(), Vec<ScopedError>> {
    let records = state::list::<TemplateRecord>(TEMPLATE_PREFIX).map_err(|error| {
        vec![ScopedError {
            scope: "templates".to_string(),
            message: error.to_string(),
        }]
    })?;

    let mut errors = vec![];
    for record in records {
        if record.value.status != TemplateStatus::Preparing {
            continue;
        }
        let name = &record.key[TEMPLATE_PREFIX.len()..];
        if let Err(error) = tick_prepare(name, &record) {
            errors.push(ScopedError {
                scope: name.to_string(),
                message: error.to_string(),
            });
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn tick_prepare(name: &str, record: &state::Versioned<TemplateRecord>) -> Result<()> {
    let id = format!("{PREPARE_PREFIX}{name}");
    for line in process::read_lines(&id).map_err(|error| anyhow!(error))? {
        info!("[{}] {}", name, line.trim());
    }
    let ProcessStatus::Exited(_) = process::status(&id).map_err(|error| anyhow!(error))? else {
        return Ok(());
    };

    // Drain what was printed right before exiting
    for line in process::read_lines(&id).map_err(|error| anyhow!(error))? {
        info!("[{}] {}", name, line.trim());
    }
    process::remove(&id).map_err(|error| anyhow!(error))?;
    info!("Prepare script of template {} has finished", name);
    state::swap(
        record,
        Some(&TemplateRecord {
            status: TemplateStatus::Ready,
            message: None,
        }),
    )?;
    Ok(())
}
