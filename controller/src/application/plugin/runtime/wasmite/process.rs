//! Host-owned child processes for wasmite plugins.
//!
//! A wasmite plugin instance only lives for one call, so it cannot own a
//! child process handle. Instead the host keeps every process in this
//! registry under an id chosen by the plugin, and the plugin drives it via
//! the `process` interface.
//!
//! The registry also records the pid of every process it starts in a file.
//! If the controller dies without cleaning up, the next start uses that
//! file to kill the processes it left behind, instead of leaving orphaned
//! game servers that keep holding ports and directories.

use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    path::{Component, Path, PathBuf},
    process::Stdio,
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::{Context as _, Result, anyhow};
use serde::{Deserialize, Serialize};
use simplelog::{debug, info, warn};
use tokio::{
    fs,
    io::{AsyncBufReadExt, AsyncRead, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, Command},
    spawn,
    sync::Mutex as AsyncMutex,
    task::JoinHandle,
    time::timeout,
};

use super::kv::write_atomically;

/// Lines kept per process when nobody drains them. Older lines are dropped,
/// so a process is never blocked on a full stdout pipe.
const LINE_BUFFER: usize = 1024;
/// How long writing to a process' stdin may take before giving up.
const WRITE_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub environment: Vec<(String, String)>,
    /// Working directory relative to the registry root.
    pub directory: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    Code(i32),
    Signal(i32),
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Running,
    Exited(Exit),
}

#[derive(Debug)]
pub enum ProcessError {
    NotFound(String),
    AlreadyExists(String),
    InvalidArgument(String),
    Io(String),
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(message)
            | Self::AlreadyExists(message)
            | Self::InvalidArgument(message)
            | Self::Io(message) => formatter.write_str(message),
        }
    }
}

type ProcessResult<T> = Result<T, ProcessError>;

struct Managed {
    spec: CommandSpec,
    child: Child,
    stdin: Option<ChildStdin>,
    lines: Arc<Mutex<VecDeque<String>>>,
    readers: Vec<JoinHandle<()>>,
}

#[derive(Serialize, Deserialize, Default)]
struct PidFile {
    #[serde(default)]
    processes: BTreeMap<String, PidEntry>,
}

#[derive(Serialize, Deserialize, Clone)]
struct PidEntry {
    pid: u32,
    program: String,
    /// Start time of the process (clock ticks since boot on Linux). Unlike
    /// the command line it survives `exec` (startup scripts `exec java`),
    /// and it differs when the kernel reuses the pid for another process.
    #[serde(default)]
    start_time: Option<u64>,
    /// Boot the start time refers to.
    #[serde(default)]
    boot_id: Option<String>,
}

pub struct ProcessRegistry {
    root: PathBuf,
    pid_file: PathBuf,
    processes: Mutex<HashMap<String, Arc<AsyncMutex<Managed>>>>,
    /// Serializes pid file writes.
    persist: AsyncMutex<()>,
}

impl ProcessRegistry {
    /// Creates a registry whose processes run relative to `root` and whose
    /// pids are recorded in `pid_file`. Kills any process a previous run
    /// recorded there and did not clean up.
    pub async fn open(root: PathBuf, pid_file: PathBuf) -> Result<Self> {
        let registry = Self {
            root,
            pid_file,
            processes: Mutex::new(HashMap::new()),
            persist: AsyncMutex::new(()),
        };
        registry.reap_orphans().await?;
        Ok(registry)
    }

    pub async fn spawn(&self, id: &str, spec: CommandSpec) -> ProcessResult<()> {
        if id.is_empty() {
            return Err(ProcessError::InvalidArgument(
                "Process id must not be empty".to_string(),
            ));
        }
        if self.lock().contains_key(id) {
            return Err(ProcessError::AlreadyExists(format!(
                "A process with id {id} already exists"
            )));
        }

        let managed = self.start(spec)?;
        debug!("Spawned process {} (pid {:?})", id, managed.child.id());

        let inserted = {
            let mut processes = self.lock();
            if processes.contains_key(id) {
                false
            } else {
                processes.insert(id.to_string(), Arc::new(AsyncMutex::new(managed)));
                true
            }
        };
        if !inserted {
            // Lost a race against a concurrent spawn with the same id. The
            // child is killed when `managed` is dropped (kill_on_drop).
            return Err(ProcessError::AlreadyExists(format!(
                "A process with id {id} already exists"
            )));
        }
        self.persist_pids().await;
        Ok(())
    }

    pub async fn respawn(&self, id: &str) -> ProcessResult<()> {
        let process = self.get(id)?;
        let mut process = process.lock().await;
        terminate_hard(&mut process).await;
        let spec = process.spec.clone();
        *process = self.start(spec)?;
        drop(process);
        self.persist_pids().await;
        Ok(())
    }

    pub async fn status(&self, id: &str) -> ProcessResult<Status> {
        let process = self.get(id)?;
        let mut process = process.lock().await;
        match process.child.try_wait() {
            Ok(None) => Ok(Status::Running),
            Ok(Some(status)) => Ok(Status::Exited(exit_of(status))),
            Err(error) => Err(ProcessError::Io(format!(
                "Failed to query process {id}: {error}"
            ))),
        }
    }

    pub async fn write(&self, id: &str, data: &[u8]) -> ProcessResult<()> {
        let process = self.get(id)?;
        let mut process = process.lock().await;
        let Some(stdin) = process.stdin.as_mut() else {
            return Err(ProcessError::Io(format!(
                "Stdin of process {id} is not available"
            )));
        };
        let write = async {
            stdin.write_all(data).await?;
            stdin.flush().await
        };
        match timeout(WRITE_TIMEOUT, write).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(error)) => Err(ProcessError::Io(format!(
                "Failed to write to process {id}: {error}"
            ))),
            Err(_) => Err(ProcessError::Io(format!(
                "Timed out writing to process {id}"
            ))),
        }
    }

    pub async fn read_lines(&self, id: &str) -> ProcessResult<Vec<String>> {
        let process = self.get(id)?;
        let process = process.lock().await;
        let mut lines = process
            .lines
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(lines.drain(..).collect())
    }

    /// Asks the process (and its process group) to exit.
    pub async fn terminate(&self, id: &str) -> ProcessResult<()> {
        let process = self.get(id)?;
        let mut process = process.lock().await;
        terminate_gracefully(&mut process.child)
            .map_err(|error| ProcessError::Io(format!("Failed to terminate {id}: {error}")))
    }

    pub async fn kill(&self, id: &str) -> ProcessResult<()> {
        let process = self.get(id)?;
        let mut process = process.lock().await;
        terminate_hard(&mut process).await;
        Ok(())
    }

    /// Kills the process if needed and forgets it.
    pub async fn remove(&self, id: &str) -> ProcessResult<()> {
        let process = self
            .lock()
            .remove(id)
            .ok_or_else(|| ProcessError::NotFound(format!("Process {id} does not exist")))?;
        terminate_hard(&mut *process.lock().await).await;
        self.persist_pids().await;
        Ok(())
    }

    pub fn list_ids(&self, prefix: &str) -> Vec<String> {
        let mut ids: Vec<String> = self
            .lock()
            .keys()
            .filter(|id| id.starts_with(prefix))
            .cloned()
            .collect();
        ids.sort_unstable();
        ids
    }

    /// Kills every process. Used when the plugin is unloaded.
    pub async fn shutdown(&self) {
        let processes: Vec<_> = self.lock().drain().collect();
        for (id, process) in processes {
            info!("Killing process {} that was still running", id);
            terminate_hard(&mut *process.lock().await).await;
        }
        self.persist_pids().await;
    }

    fn get(&self, id: &str) -> ProcessResult<Arc<AsyncMutex<Managed>>> {
        self.lock()
            .get(id)
            .cloned()
            .ok_or_else(|| ProcessError::NotFound(format!("Process {id} does not exist")))
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, Arc<AsyncMutex<Managed>>>> {
        self.processes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn start(&self, spec: CommandSpec) -> ProcessResult<Managed> {
        let directory = resolve_directory(&self.root, &spec.directory)?;

        let mut command = Command::new(&spec.program);
        command
            .args(&spec.args)
            .envs(spec.environment.iter().map(|(key, value)| (key, value)))
            .current_dir(&directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        // Run every process in its own process group: signals then reach
        // the whole tree (e.g. bash and the java process it started), and a
        // Ctrl+C in the controller's terminal is not forwarded to it.
        #[cfg(unix)]
        command.process_group(0);

        let mut child = command.spawn().map_err(|error| {
            ProcessError::Io(format!("Failed to spawn {}: {error}", spec.program))
        })?;

        let lines = Arc::new(Mutex::new(VecDeque::new()));
        let mut readers = vec![];
        if let Some(stdout) = child.stdout.take() {
            readers.push(spawn(read_into(stdout, lines.clone())));
        }
        if let Some(stderr) = child.stderr.take() {
            readers.push(spawn(read_into(stderr, lines.clone())));
        }

        Ok(Managed {
            spec,
            stdin: child.stdin.take(),
            child,
            lines,
            readers,
        })
    }

    async fn persist_pids(&self) {
        let _guard = self.persist.lock().await;
        let mut file = PidFile::default();
        let processes: Vec<_> = self
            .lock()
            .iter()
            .map(|(id, process)| (id.clone(), process.clone()))
            .collect();
        for (id, process) in processes {
            let process = process.lock().await;
            if let Some(pid) = process.child.id() {
                file.processes.insert(
                    id,
                    PidEntry {
                        pid,
                        program: process.spec.program.clone(),
                        start_time: start_time(pid),
                        boot_id: boot_id(),
                    },
                );
            }
        }
        let result = match toml::to_string(&file) {
            Ok(data) => write_atomically(&self.pid_file, data.as_bytes()).await,
            Err(error) => Err(error.into()),
        };
        if let Err(error) = result {
            warn!(
                "Failed to record process ids in {}: {}",
                self.pid_file.display(),
                error
            );
        }
    }

    async fn reap_orphans(&self) -> Result<()> {
        if !self.pid_file.exists() {
            return Ok(());
        }
        let data = fs::read_to_string(&self.pid_file).await?;
        let file: PidFile = toml::from_str(&data)
            .with_context(|| format!("Failed to parse {}", self.pid_file.display()))?;
        for (id, entry) in file.processes {
            if kill_orphan(&entry) {
                warn!(
                    "Killed process {} (pid {}) left behind by a previous controller run",
                    id, entry.pid
                );
            }
        }
        write_atomically(
            &self.pid_file,
            toml::to_string(&PidFile::default())?.as_bytes(),
        )
        .await
    }
}

/// Resolves a plugin-supplied relative directory below `root`, rejecting
/// absolute paths and `..` so a typo cannot point a process elsewhere.
fn resolve_directory(root: &Path, directory: &str) -> ProcessResult<PathBuf> {
    let relative = Path::new(directory);
    if relative
        .components()
        .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return Err(ProcessError::InvalidArgument(format!(
            "Directory {directory} must be relative to the plugin data directory and must not contain '..'"
        )));
    }
    let resolved = root.join(relative);
    if !resolved.is_dir() {
        return Err(ProcessError::NotFound(format!(
            "Directory {directory} does not exist"
        )));
    }
    Ok(resolved)
}

async fn read_into<R: AsyncRead + Unpin>(reader: R, lines: Arc<Mutex<VecDeque<String>>>) {
    let mut reader = BufReader::new(reader);
    let mut buffer = Vec::new();
    loop {
        buffer.clear();
        match reader.read_until(b'\n', &mut buffer).await {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                let line = String::from_utf8_lossy(&buffer)
                    .trim_end_matches(['\r', '\n'])
                    .to_string();
                let mut lines = lines
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if lines.len() >= LINE_BUFFER {
                    lines.pop_front();
                }
                lines.push_back(line);
            }
        }
    }
}

async fn terminate_hard(process: &mut Managed) {
    if matches!(process.child.try_wait(), Ok(Some(_))) {
        return;
    }
    #[cfg(unix)]
    if let Some(pid) = process.child.id() {
        signal_group(pid, libc::SIGKILL);
    }
    if let Err(error) = process.child.kill().await {
        debug!("Failed to kill process: {}", error);
    }
    for reader in &process.readers {
        reader.abort();
    }
}

#[cfg(unix)]
#[allow(clippy::unnecessary_wraps)] // Must match the fallible non-unix version
fn terminate_gracefully(child: &mut Child) -> std::io::Result<()> {
    match child.id() {
        Some(pid) => {
            signal_group(pid, libc::SIGTERM);
            Ok(())
        }
        // Already exited and reaped
        None => Ok(()),
    }
}

#[cfg(not(unix))]
fn terminate_gracefully(child: &mut Child) -> std::io::Result<()> {
    child.start_kill()
}

#[cfg(unix)]
fn signal_group(pid: u32, signal: libc::c_int) {
    let Ok(pid) = libc::pid_t::try_from(pid) else {
        return;
    };
    // SAFETY: kill(2) has no memory safety requirements. A negative pid
    // addresses the process group, which equals the pid because every
    // child is started with process_group(0).
    unsafe {
        libc::kill(-pid, signal);
    }
}

/// Kills the process group of a process recorded by a previous run, but
/// only if it provably is the process we started: pids get reused, and
/// killing an unrelated process would be far worse than leaving an orphan.
#[cfg(target_os = "linux")]
fn kill_orphan(entry: &PidEntry) -> bool {
    let (Some(recorded), Some(recorded_boot)) = (entry.start_time, entry.boot_id.as_ref()) else {
        warn!(
            "Process {} ({}) may have been left behind by a previous run, but it cannot be verified; stop it manually",
            entry.pid, entry.program
        );
        return false;
    };
    if boot_id().as_ref() != Some(recorded_boot) {
        // The machine rebooted, so nothing from the previous run is alive
        return false;
    }
    let is_ours = match start_time(entry.pid) {
        Some(current) => current == recorded,
        // The group leader is gone, but members of its group (e.g. a server
        // started in the background by a script) may still run. A process
        // group id cannot be handed out again while the group has members,
        // so any process still in it belongs to us.
        None => group_has_members(entry.pid),
    };
    if is_ours {
        signal_group(entry.pid, libc::SIGKILL);
    }
    is_ours
}

#[cfg(not(target_os = "linux"))]
fn kill_orphan(entry: &PidEntry) -> bool {
    warn!(
        "Process {} ({}) may have been left behind by a previous run; stop it manually",
        entry.pid, entry.program
    );
    false
}

/// Process group and start time from `/proc/<pid>/stat`.
#[cfg(target_os = "linux")]
fn proc_stat(pid: &str) -> Option<(u32, u64)> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // The command name in parentheses may contain spaces, so split after it.
    // fields[0] is field 3 of proc_pid_stat(5), so field n is fields[n - 3].
    let fields: Vec<&str> = stat
        .get(stat.rfind(')')? + 1..)?
        .split_whitespace()
        .collect();
    Some((fields.get(2)?.parse().ok()?, fields.get(19)?.parse().ok()?))
}

#[cfg(target_os = "linux")]
fn start_time(pid: u32) -> Option<u64> {
    proc_stat(&pid.to_string()).map(|(_, start)| start)
}

#[cfg(not(target_os = "linux"))]
fn start_time(_pid: u32) -> Option<u64> {
    None
}

#[cfg(target_os = "linux")]
fn boot_id() -> Option<String> {
    std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
        .ok()
        .map(|id| id.trim().to_string())
}

#[cfg(not(target_os = "linux"))]
fn boot_id() -> Option<String> {
    None
}

#[cfg(target_os = "linux")]
fn group_has_members(group: u32) -> bool {
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return false;
    };
    entries.flatten().any(|entry| {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        name.bytes().all(|byte| byte.is_ascii_digit())
            && proc_stat(&name).is_some_and(|(pgrp, _)| pgrp == group)
    })
}

#[cfg(unix)]
fn exit_of(status: std::process::ExitStatus) -> Exit {
    use std::os::unix::process::ExitStatusExt;
    if let Some(code) = status.code() {
        Exit::Code(code)
    } else if let Some(signal) = status.signal() {
        Exit::Signal(signal)
    } else {
        Exit::Unknown
    }
}

#[cfg(not(unix))]
fn exit_of(status: std::process::ExitStatus) -> Exit {
    status.code().map_or(Exit::Unknown, Exit::Code)
}

impl From<ProcessError> for anyhow::Error {
    fn from(error: ProcessError) -> Self {
        anyhow!(error.to_string())
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::time::Instant;

    use super::*;

    fn sh(script: &str) -> CommandSpec {
        CommandSpec {
            program: "sh".to_string(),
            args: vec!["-c".to_string(), script.to_string()],
            environment: vec![("GREETING".to_string(), "hello".to_string())],
            directory: String::new(),
        }
    }

    async fn registry() -> (tempfile::TempDir, ProcessRegistry) {
        let directory = tempfile::tempdir().unwrap();
        let registry = ProcessRegistry::open(
            directory.path().to_path_buf(),
            directory.path().join(".wasmite").join("processes.toml"),
        )
        .await
        .unwrap();
        (directory, registry)
    }

    async fn wait_for<F: FnMut() -> bool>(mut condition: F) {
        let start = Instant::now();
        while !condition() {
            assert!(
                start.elapsed() < Duration::from_secs(10),
                "condition not met in time"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    async fn collect_until(registry: &ProcessRegistry, id: &str, needle: &str) -> Vec<String> {
        let start = Instant::now();
        let mut all = vec![];
        loop {
            all.extend(registry.read_lines(id).await.unwrap());
            if all.iter().any(|line| line == needle) {
                return all;
            }
            assert!(
                start.elapsed() < Duration::from_secs(10),
                "never saw {needle:?}, got {all:?}"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    async fn wait_exit(registry: &ProcessRegistry, id: &str) -> Exit {
        let start = Instant::now();
        loop {
            if let Status::Exited(exit) = registry.status(id).await.unwrap() {
                return exit;
            }
            assert!(start.elapsed() < Duration::from_secs(10), "did not exit");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }

    #[tokio::test]
    async fn stdin_stdout_and_exit_code() {
        let (_directory, registry) = registry().await;
        registry
            .spawn("p", sh("echo $GREETING; read line; echo got $line; exit 3"))
            .await
            .unwrap();
        collect_until(&registry, "p", "hello").await;
        registry.write("p", b"world\n").await.unwrap();
        collect_until(&registry, "p", "got world").await;
        assert_eq!(wait_exit(&registry, "p").await, Exit::Code(3));
    }

    #[tokio::test]
    async fn duplicate_ids_are_rejected() {
        let (_directory, registry) = registry().await;
        registry.spawn("p", sh("sleep 5")).await.unwrap();
        assert!(matches!(
            registry.spawn("p", sh("true")).await,
            Err(ProcessError::AlreadyExists(_))
        ));
        assert!(matches!(
            registry.status("missing").await,
            Err(ProcessError::NotFound(_))
        ));
        registry.remove("p").await.unwrap();
        assert_eq!(registry.list_ids(""), [] as [std::string::String; 0]);
    }

    #[tokio::test]
    async fn terminate_reaches_the_process_group() {
        let (_directory, registry) = registry().await;
        // The shell runs sleep as a child, so only a signal to the whole
        // group stops it promptly.
        registry
            .spawn("p", sh("trap 'exit 7' TERM; sleep 30 & wait"))
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(200)).await;
        registry.terminate("p").await.unwrap();
        assert_eq!(wait_exit(&registry, "p").await, Exit::Code(7));
    }

    #[tokio::test]
    async fn respawn_restarts_with_same_command() {
        let (_directory, registry) = registry().await;
        registry
            .spawn("p", sh("echo started; sleep 30"))
            .await
            .unwrap();
        collect_until(&registry, "p", "started").await;
        registry.respawn("p").await.unwrap();
        collect_until(&registry, "p", "started").await;
        assert_eq!(registry.status("p").await.unwrap(), Status::Running);
        registry.kill("p").await.unwrap();
        assert!(matches!(
            registry.status("p").await.unwrap(),
            Status::Exited(_)
        ));
    }

    #[tokio::test]
    async fn rejects_directories_outside_root() {
        let (directory, registry) = registry().await;
        for bad in ["..", "../x", "/tmp", "a/../../b"] {
            let mut spec = sh("true");
            spec.directory = bad.to_string();
            assert!(
                matches!(
                    registry.spawn("p", spec).await,
                    Err(ProcessError::InvalidArgument(_))
                ),
                "{bad} was accepted"
            );
        }
        std::fs::create_dir(directory.path().join("sub")).unwrap();
        let mut spec = sh("pwd");
        spec.directory = "sub".to_string();
        registry.spawn("p", spec).await.unwrap();
        assert_eq!(wait_exit(&registry, "p").await, Exit::Code(0));
        tokio::time::sleep(Duration::from_millis(100)).await;
        let lines = registry.read_lines("p").await.unwrap();
        assert!(
            lines.iter().any(|line| line.ends_with("/sub")),
            "unexpected working directory: {lines:?}"
        );
    }

    #[tokio::test]
    async fn output_buffer_is_bounded() {
        let (_directory, registry) = registry().await;
        registry
            .spawn(
                "p",
                sh("i=0; while [ $i -lt 3000 ]; do echo $i; i=$((i+1)); done"),
            )
            .await
            .unwrap();
        // Not draining while it runs must not block the process
        assert_eq!(wait_exit(&registry, "p").await, Exit::Code(0));
        tokio::time::sleep(Duration::from_millis(100)).await;
        let lines = registry.read_lines("p").await.unwrap();
        assert_eq!(lines.len(), LINE_BUFFER);
        assert_eq!(lines.last().map(String::as_str), Some("2999"));
    }

    #[tokio::test]
    async fn orphans_of_a_previous_run_are_killed() {
        let directory = tempfile::tempdir().unwrap();
        let pid_file = directory.path().join("processes.toml");
        let pid = {
            let registry = ProcessRegistry::open(directory.path().to_path_buf(), pid_file.clone())
                .await
                .unwrap();
            // `exec` replaces the shell, like startup scripts that `exec java`
            registry.spawn("p", sh("exec sleep 30")).await.unwrap();
            let pid = registry.get("p").unwrap().lock().await.child.id().unwrap();
            // Simulate a crash: the registry disappears without cleanup and
            // kill_on_drop never runs.
            std::mem::forget(registry);
            pid
        };
        assert!(Path::new(&format!("/proc/{pid}")).exists());

        let _registry = ProcessRegistry::open(directory.path().to_path_buf(), pid_file.clone())
            .await
            .unwrap();
        wait_for(|| {
            // The killed child stays a zombie until reaped by its parent
            // (this test process), so check its state instead of existence.
            std::fs::read_to_string(format!("/proc/{pid}/stat"))
                .map_or(true, |stat| stat.contains(") Z "))
        })
        .await;
        let recorded = std::fs::read_to_string(pid_file).unwrap();
        assert!(!recorded.contains(&pid.to_string()));
    }

    #[tokio::test]
    async fn orphaned_group_members_are_killed_when_the_leader_is_gone() {
        let directory = tempfile::tempdir().unwrap();
        let pid_file = directory.path().join("processes.toml");
        let leader = {
            let registry = ProcessRegistry::open(directory.path().to_path_buf(), pid_file.clone())
                .await
                .unwrap();
            // The shell starts sleep in the background and exits right away
            registry
                .spawn("p", sh("sleep 30 & echo started"))
                .await
                .unwrap();
            collect_until(&registry, "p", "started").await;
            let leader = registry.get("p").unwrap().lock().await.child.id().unwrap();
            assert_eq!(wait_exit(&registry, "p").await, Exit::Code(0));
            std::mem::forget(registry);
            leader
        };
        let alive_members = || {
            std::fs::read_dir("/proc")
                .unwrap()
                .flatten()
                .filter_map(|entry| std::fs::read_to_string(entry.path().join("stat")).ok())
                .filter(|stat| !stat.contains(") Z "))
                .filter_map(|stat| proc_stat_of(&stat))
                .filter(|pgrp| *pgrp == leader)
                .count()
        };
        assert_eq!(alive_members(), 1, "sleep should still run in the group");

        let _registry = ProcessRegistry::open(directory.path().to_path_buf(), pid_file)
            .await
            .unwrap();
        wait_for(|| alive_members() == 0).await;
    }

    fn proc_stat_of(stat: &str) -> Option<u32> {
        let fields: Vec<&str> = stat
            .get(stat.rfind(')')? + 1..)?
            .split_whitespace()
            .collect();
        fields.get(2)?.parse().ok()
    }

    #[tokio::test]
    async fn unrelated_pids_are_not_killed() {
        let directory = tempfile::tempdir().unwrap();
        let pid_file = directory.path().join("processes.toml");
        let mut innocent = Command::new("sleep").arg("30").spawn().unwrap();
        let pid = innocent.id().unwrap();
        let boot = boot_id().unwrap();
        // Same pid, but a different start time (the pid was reused), and an
        // entry from an older release without a start time
        std::fs::write(
            &pid_file,
            format!(
                "[processes.reused]\npid = {pid}\nprogram = \"sleep\"\nstart_time = 1\nboot_id = \"{boot}\"\n\n[processes.legacy]\npid = {pid}\nprogram = \"sleep\"\n"
            ),
        )
        .unwrap();
        let _registry = ProcessRegistry::open(directory.path().to_path_buf(), pid_file)
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(innocent.try_wait().unwrap().is_none(), "sleep was killed");
        innocent.kill().await.unwrap();
    }
}
