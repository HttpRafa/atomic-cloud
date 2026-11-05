//! Server lifecycle. Each server has a [`ServerRecord`] in the kv store and
//! a host process with the id `server/<uuid>`.
//!
//! Every state change of a record is a compare-and-swap against the version
//! that was read, so a tick and a concurrent stop/restart call can never
//! overwrite each other: whichever loses simply sees the new state on the
//! next tick.

use std::time::Duration;

use anyhow::{Result, anyhow, bail};
use common::name::TimedName;

use crate::{
    config::Config,
    debug,
    generated::atomic::plugin_system::{
        errors::{Error, ScopedError},
        fs::remove_dir_all,
        platform::get_certificate,
        process::{self, ProcessStatus},
        server::stopped,
        types::{DiskRetention, Server},
    },
    info,
    state::{self, NodeRecord, ServerRecord, ServerState, Versioned},
    storage::Storage,
    template::Template,
    warn,
};

/* Variables */
const CONTROLLER_ADDRESS: &str = "CONTROLLER_ADDRESS";
const CONTROLLER_CERTIFICATE: &str = "CONTROLLER_CERTIFICATE";
const SERVER_TOKEN: &str = "SERVER_TOKEN";
const SERVER_MEMORY: &str = "SERVER_MEMORY";
const SERVER_PORT: &str = "SERVER_PORT";

fn process_id(uuid: &str) -> String {
    format!("server/{uuid}")
}

/// Forgets all servers of a previous controller run and removes their
/// temporary directories. Their processes are already gone: the host kills
/// leftovers before the plugin is initialized.
pub fn reset() -> Result<()> {
    for record in state::list::<ServerRecord>(state::SERVER_PREFIX)? {
        state::delete(&record.key)?;
    }
    state::delete(state::PORTS_KEY)?;
    if Storage::guest(&Storage::temporary_directory()).exists() {
        info!("Removing temporary files");
        remove_dir_all(&Storage::temporary_directory()).map_err(|error| anyhow!(error))?;
    }
    Ok(())
}

pub fn start(node: &str, server: &Server) -> Result<String> {
    let template = Template::load_ready(&server.allocation.specification.image)?;
    let Some(node_record) = state::get::<NodeRecord>(&state::node_key(node))? else {
        bail!(Error::NotFound(format!("Node {node} is not initialized")));
    };

    let retention = server.allocation.specification.disk_retention;
    let name =
        TimedName::new_no_identifier(&server.name, matches!(retention, DiskRetention::Permanent));
    let directory = Storage::server_directory(node, name.get_name(), retention);

    // Prepare the directory
    let guest_directory = Storage::guest(&directory);
    if !guest_directory.exists() {
        template.copy_to(&guest_directory)?;
    }

    // Prepare the environment
    let mut environment = server.allocation.specification.environment.clone();
    environment.push((CONTROLLER_ADDRESS.to_string(), node_record.controller));
    environment.push((SERVER_TOKEN.to_string(), server.token.clone()));
    environment.push((
        SERVER_MEMORY.to_string(),
        server.allocation.resources.memory.to_string(),
    ));
    environment.push((
        SERVER_PORT.to_string(),
        server
            .allocation
            .ports
            .first()
            .ok_or(anyhow!("No ports allocated"))?
            .port
            .to_string(),
    ));
    if let Some(certificate) = get_certificate() {
        environment.push((CONTROLLER_CERTIFICATE.to_string(), certificate));
    }

    let record = ServerRecord {
        uuid: server.uuid.clone(),
        name: name.get_name().to_string(),
        template: template.name().to_string(),
        directory: directory.clone(),
        temporary: matches!(retention, DiskRetention::Temporary),
        state: ServerState::Running,
        since: state::now(),
    };
    let key = state::server_key(node, &server.uuid);
    if state::get::<ServerRecord>(&key)?.is_some() {
        bail!(Error::AlreadyExists(format!(
            "Server {} is already running",
            server.name
        )));
    }

    let id = process_id(&server.uuid);
    process::spawn(&id, &template.startup_command(&directory, environment)?)
        .map_err(|error| anyhow!(error))?;
    state::put(&key, &record)?;

    info!("Server {} started", server.name);
    Ok(id)
}

pub fn restart(node: &str, server: &Server) -> Result<()> {
    let key = state::server_key(node, &server.uuid);
    let template = transition(&key, ServerState::Restarting)?;
    send_shutdown(&template, &server.uuid);
    Ok(())
}

pub fn stop(node: &str, server: &Server) -> Result<()> {
    let key = state::server_key(node, &server.uuid);
    if state::get::<ServerRecord>(&key)?.is_none() {
        // Nothing to stop (e.g. it crashed and was already cleaned up)
        stopped(&server.uuid);
        return Ok(());
    }
    let template = transition(&key, ServerState::Stopping)?;
    send_shutdown(&template, &server.uuid);
    Ok(())
}

/// Moves the record at `key` into `next` and returns its template.
fn transition(key: &str, next: ServerState) -> Result<Template> {
    let record = state::update(key, |record: Option<ServerRecord>| {
        let Some(mut record) = record else {
            bail!(Error::NotFound(format!("Server {key} does not exist")));
        };
        record.state = next;
        record.since = state::now();
        Ok((Some(record.clone()), record))
    })?;
    Template::load(&record.template)
}

/// Asks the server process to shut down: with the template's shutdown
/// command if it has one, otherwise with a termination signal. A failure is
/// only logged: the tick kills the process once the timeout runs out.
fn send_shutdown(template: &Template, uuid: &str) {
    let id = process_id(uuid);
    let result = match template.shutdown() {
        Some(command) => process::write(&id, format!("{command}\n").as_bytes()),
        None => process::terminate(&id),
    };
    if let Err(error) = result {
        warn!(
            "Failed to send shutdown command to server {}: {}",
            uuid, error
        );
    }
}

pub fn tick(node: &str, config: &Config) -> Result<(), Vec<ScopedError>> {
    let records = state::list::<ServerRecord>(&state::server_prefix(node)).map_err(|error| {
        vec![ScopedError {
            scope: node.to_string(),
            message: error.to_string(),
        }]
    })?;

    let mut errors = vec![];
    for record in records {
        if let Err(error) = tick_server(config, &record) {
            errors.push(ScopedError {
                scope: record.value.name.clone(),
                message: error.to_string(),
            });
            if let Err(error) = cleanup(&record) {
                errors.push(ScopedError {
                    scope: record.value.name.clone(),
                    message: error.to_string(),
                });
            }
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn tick_server(config: &Config, record: &Versioned<ServerRecord>) -> Result<()> {
    let server = &record.value;
    let id = process_id(&server.uuid);
    let elapsed = Duration::from_secs(state::now().saturating_sub(server.since));

    match server.state {
        ServerState::Running => Ok(()),
        ServerState::Restarting => {
            let exited = matches!(
                process::status(&id).map_err(|error| anyhow!(error))?,
                ProcessStatus::Exited(_)
            );
            if !exited && elapsed <= *config.restart_timeout() {
                return Ok(());
            }
            if exited {
                info!("Server {} exited, starting it again", server.name);
            } else {
                warn!(
                    "Server {} failed to stop in {:?}. Killing and respawning process...",
                    server.name,
                    config.restart_timeout()
                );
            }
            let mut running = server.clone();
            running.state = ServerState::Running;
            running.since = state::now();
            // Only respawn if no stop came in meanwhile
            if state::swap(record, Some(&running))? {
                process::respawn(&id).map_err(|error| anyhow!(error))?;
            }
            Ok(())
        }
        ServerState::Stopping => {
            let exited = matches!(
                process::status(&id).map_err(|error| anyhow!(error))?,
                ProcessStatus::Exited(_)
            );
            if !exited {
                if elapsed <= *config.stop_timeout() {
                    return Ok(());
                }
                warn!(
                    "Server {} failed to stop in {:?}. Killing process...",
                    server.name,
                    config.stop_timeout()
                );
                if let Err(error) = process::kill(&id) {
                    warn!("Failed to kill process: {}", error);
                }
            }
            info!("Server {} stopped.", server.name);
            cleanup(record)
        }
    }
}

/// Removes the process, the temporary directory and the record of a server
/// and reports it as stopped.
fn cleanup(record: &Versioned<ServerRecord>) -> Result<()> {
    let server = &record.value;
    debug!("Cleaning up server {}", server.name);
    match process::remove(&process_id(&server.uuid)) {
        Ok(()) | Err(Error::NotFound(_)) => {}
        Err(error) => return Err(anyhow!(error)),
    }
    if server.temporary {
        remove_dir_all(&server.directory).map_err(|error| anyhow!(error))?;
    }
    state::delete(&record.key)?;
    stopped(&server.uuid);
    debug!("Cleaned up server {}", server.name);
    Ok(())
}
