//! Implementations of the interfaces wasmite plugins import.

use std::path::{Component, Path};

use simplelog::{debug, error, info, warn};
use tokio::fs;

use crate::application::plugin::runtime::config::Permissions;

use super::{
    State,
    generated::atomic::plugin_system::{
        errors::{self, Error},
        fs as plugin_fs, kv, log, platform,
        process::{self, Command, ExitStatus, ProcessStatus},
        server, types,
    },
    process::{CommandSpec, Exit, ProcessError, Status},
};

type Fallible<T> = wasmtime::Result<Result<T, Error>>;

impl errors::Host for State {}
impl types::Host for State {}

impl log::Host for State {
    async fn log(&mut self, level: log::Level, message: String) -> wasmtime::Result<()> {
        let name = self.context.name.to_uppercase();
        match level {
            log::Level::Debug => {
                debug!("[{}] {}", name, message);
            }
            log::Level::Info => {
                info!("[{}] {}", name, message);
            }
            log::Level::Warn => {
                warn!("[{}] {}", name, message);
            }
            log::Level::Error => {
                error!("[{}] {}", name, message);
            }
        }
        Ok(())
    }
}

impl platform::Host for State {
    async fn get_os(&mut self) -> wasmtime::Result<platform::Os> {
        Ok(if cfg!(target_os = "windows") {
            platform::Os::Windows
        } else {
            platform::Os::Unix
        })
    }

    async fn get_certificate(&mut self) -> wasmtime::Result<Option<String>> {
        Ok(self.context.certificate.clone())
    }
}

impl kv::Host for State {
    async fn get(&mut self, key: String) -> Fallible<Option<Vec<u8>>> {
        Ok(Ok(self.context.kv.get(&key)))
    }

    async fn set(&mut self, key: String, value: Vec<u8>) -> Fallible<()> {
        Ok(self.context.kv.set(&key, value).await.map_err(kv_error))
    }

    async fn delete(&mut self, key: String) -> Fallible<()> {
        Ok(self.context.kv.delete(&key).await.map_err(kv_error))
    }

    async fn compare_and_swap(
        &mut self,
        key: String,
        expected: Option<Vec<u8>>,
        new: Option<Vec<u8>>,
    ) -> Fallible<bool> {
        Ok(self
            .context
            .kv
            .compare_and_swap(&key, expected.as_deref(), new)
            .await
            .map_err(kv_error))
    }

    async fn keys(&mut self, prefix: String) -> Fallible<Vec<String>> {
        Ok(Ok(self.context.kv.keys(&prefix)))
    }
}

impl process::Host for State {
    async fn spawn(&mut self, id: String, command: Command) -> Fallible<()> {
        if let Err(error) = self.require(&Permissions::ALLOW_PROCESS, "spawn processes") {
            return Ok(Err(error));
        }
        let spec = CommandSpec {
            program: command.program,
            args: command.args,
            environment: command.environment,
            directory: command.directory,
        };
        Ok(self
            .context
            .processes
            .spawn(&id, spec)
            .await
            .map_err(Into::into))
    }

    async fn respawn(&mut self, id: String) -> Fallible<()> {
        if let Err(error) = self.require(&Permissions::ALLOW_PROCESS, "spawn processes") {
            return Ok(Err(error));
        }
        Ok(self
            .context
            .processes
            .respawn(&id)
            .await
            .map_err(Into::into))
    }

    async fn status(&mut self, id: String) -> Fallible<ProcessStatus> {
        Ok(self
            .context
            .processes
            .status(&id)
            .await
            .map(Into::into)
            .map_err(Into::into))
    }

    async fn write(&mut self, id: String, data: Vec<u8>) -> Fallible<()> {
        Ok(self
            .context
            .processes
            .write(&id, &data)
            .await
            .map_err(Into::into))
    }

    async fn read_lines(&mut self, id: String) -> Fallible<Vec<String>> {
        Ok(self
            .context
            .processes
            .read_lines(&id)
            .await
            .map_err(Into::into))
    }

    async fn terminate(&mut self, id: String) -> Fallible<()> {
        Ok(self
            .context
            .processes
            .terminate(&id)
            .await
            .map_err(Into::into))
    }

    async fn kill(&mut self, id: String) -> Fallible<()> {
        Ok(self.context.processes.kill(&id).await.map_err(Into::into))
    }

    async fn remove(&mut self, id: String) -> Fallible<()> {
        Ok(self.context.processes.remove(&id).await.map_err(Into::into))
    }

    async fn list_ids(&mut self, prefix: String) -> wasmtime::Result<Vec<String>> {
        Ok(self.context.processes.list_ids(&prefix))
    }
}

impl plugin_fs::Host for State {
    async fn remove_dir_all(&mut self, path: String) -> Fallible<()> {
        if let Err(error) = self.require(&Permissions::ALLOW_REMOVE_DIR_ALL, "remove directories") {
            return Ok(Err(error));
        }
        let relative = Path::new(&path);
        if path.is_empty()
            || relative
                .components()
                .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
        {
            return Ok(Err(Error::InvalidArgument(format!(
                "Path {path} must be a non-empty path relative to the plugin data directory"
            ))));
        }
        let target = self.context.data_directory.join(relative);
        match fs::remove_dir_all(&target).await {
            Ok(()) => Ok(Ok(())),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Ok(())),
            Err(error) => Ok(Err(Error::Io(format!("Failed to remove {path}: {error}")))),
        }
    }
}

impl server::Host for State {
    async fn stopped(&mut self, server: String) -> wasmtime::Result<()> {
        self.context.release_guard(&server);
        Ok(())
    }
}

impl State {
    fn require(&self, permission: &Permissions, action: &str) -> Result<(), Error> {
        if self.context.permissions.contains(permission.clone()) {
            Ok(())
        } else {
            Err(Error::Unauthorized(format!(
                "Plugin {} is not allowed to {action}; grant {permission:?} in wasm-plugins.toml",
                self.context.name
            )))
        }
    }
}

#[allow(clippy::needless_pass_by_value)]
fn kv_error(error: anyhow::Error) -> Error {
    Error::KeyValue(error.to_string())
}

impl From<ProcessError> for Error {
    fn from(error: ProcessError) -> Self {
        match error {
            ProcessError::NotFound(message) => Error::NotFound(message),
            ProcessError::AlreadyExists(message) => Error::AlreadyExists(message),
            ProcessError::InvalidArgument(message) => Error::InvalidArgument(message),
            ProcessError::Io(message) => Error::Io(message),
        }
    }
}

impl From<Status> for ProcessStatus {
    fn from(status: Status) -> Self {
        match status {
            Status::Running => ProcessStatus::Running,
            Status::Exited(Exit::Code(code)) => ProcessStatus::Exited(ExitStatus::Code(code)),
            Status::Exited(Exit::Signal(signal)) => {
                ProcessStatus::Exited(ExitStatus::Signal(signal))
            }
            Status::Exited(Exit::Unknown) => ProcessStatus::Exited(ExitStatus::Unknown),
        }
    }
}
