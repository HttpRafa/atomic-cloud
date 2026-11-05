//! End-to-end tests that load the real local plugin (`local.wasm`) into the
//! wasmite runtime and drive servers made from a tiny `sh` based template.
//!
//! Build the plugin first:
//!
//! ```sh
//! RUSTFLAGS="-Z wasi-exec-model=reactor" cargo build -p local --target wasm32-wasip2
//! ```
//!
//! The tests are skipped when the plugin is missing, unless
//! `ATOMIC_REQUIRE_PLUGIN_TESTS` is set (as in CI), in which case they fail.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use tempfile::TempDir;

use crate::application::{
    plugin::{
        GenericPlugin,
        runtime::{config::Permissions, epoch::EpochInvoker},
    },
    server::{
        guard::{Guard, WeakGuard},
        screen::GenericScreen,
    },
};

use super::{
    Runtime,
    generated::atomic::plugin_system::{
        errors::{Error, ScopedError},
        types::{
            Address, Allocation, Capabilities, DiskRetention, Resources, Server, ServerProposal,
            Specification,
        },
    },
    init::{LoadOptions, load},
    plugin::Plugin,
    screen::Screen,
};

const NODE: &str = "node-1";
const WAIT: Duration = Duration::from_secs(15);

const CONFIG: &str = r#"
[network]
host = "127.0.0.1"

[network.ports]
range = { start = 40000, end = 40010 }

[network.ports.mappings]
lobby = [40005]

[timeouts.stop]
secs = 2
nanos = 0

[timeouts.restart]
secs = 2
nanos = 0
"#;

/// Echoes its input, and exits on "stop".
const ECHO_STARTUP: &str = r#"echo "started on $SERVER_PORT with $GREETING"; while read line; do if [ "$line" = stop ]; then echo bye; exit 0; fi; echo "got $line"; done"#;

fn plugin_path() -> Option<PathBuf> {
    let path = std::env::var_os("ATOMIC_LOCAL_WASM").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/wasm32-wasip2/debug/local.wasm"),
        PathBuf::from,
    );
    if path.exists() {
        Some(path)
    } else if std::env::var_os("ATOMIC_REQUIRE_PLUGIN_TESTS").is_some() {
        panic!("local plugin not found at {}", path.display());
    } else {
        eprintln!(
            "skipping: local plugin not built ({} missing)",
            path.display()
        );
        None
    }
}

struct Harness {
    directory: TempDir,
    plugin: Plugin,
}

struct TemplateSpec<'a> {
    name: &'a str,
    shutdown: Option<&'a str>,
    prepare: &'a str,
    startup: &'a str,
    exclusions: &'a [&'a str],
}

impl Default for TemplateSpec<'_> {
    fn default() -> Self {
        Self {
            name: "echo",
            shutdown: Some("stop"),
            prepare: "echo preparing; echo prepared > prepared.txt",
            startup: ECHO_STARTUP,
            exclusions: &["template.toml"],
        }
    }
}

impl Harness {
    async fn new(templates: &[TemplateSpec<'_>]) -> Option<Self> {
        let source = std::fs::read(plugin_path()?).unwrap();
        let directory = tempfile::tempdir().unwrap();
        let data = directory.path().join("data");
        let configs = directory.path().join("configs");
        std::fs::create_dir_all(&configs).unwrap();
        std::fs::write(configs.join("config.toml"), CONFIG).unwrap();
        for template in templates {
            write_template(&data, template);
        }

        let plugin = load(
            LoadOptions {
                name: "local".to_string(),
                identifier: "test".to_string(),
                certificate: None,
                permissions: Permissions::ALLOW_PROCESS | Permissions::ALLOW_REMOVE_DIR_ALL,
                mounts: vec![],
                data_directory: data,
                config_directory: configs,
                cache: false,
            },
            &source,
            &mut EpochInvoker::new(),
        )
        .await
        .unwrap();

        let harness = Self { directory, plugin };
        let caps = Capabilities {
            memory: None,
            max_servers: None,
            child: None,
        };
        harness
            .runtime()
            .call(async |bindings, store| {
                bindings
                    .atomic_plugin_system_node()
                    .call_init(store, NODE, &caps, "https://localhost:8080")
                    .await
            })
            .await
            .unwrap()
            .unwrap();
        Some(harness)
    }

    fn runtime(&self) -> &Arc<Runtime> {
        self.plugin.runtime()
    }

    fn data(&self) -> PathBuf {
        self.directory.path().join("data")
    }

    fn kv(&self, key: &str) -> Option<String> {
        self.runtime()
            .context()
            .kv
            .get(key)
            .map(|value| String::from_utf8(value).unwrap())
    }

    async fn tick(&self) {
        self.plugin.tick().await.unwrap().unwrap();
        let result = self
            .runtime()
            .call(async |bindings, store| {
                bindings
                    .atomic_plugin_system_node()
                    .call_tick(store, NODE)
                    .await
            })
            .await
            .unwrap();
        if let Err(errors) = result {
            panic!("node tick failed: {}", format_errors(&errors));
        }
    }

    /// Ticks until `condition` holds.
    async fn tick_until<F: FnMut(&Self) -> bool>(&self, what: &str, mut condition: F) {
        let start = Instant::now();
        loop {
            self.tick().await;
            if condition(self) {
                return;
            }
            assert!(start.elapsed() < WAIT, "timed out waiting for {what}");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    async fn wait_template(&self, name: &str, status: &str) {
        let key = format!("template/{name}");
        self.tick_until(&format!("template {name} to be {status}"), |harness| {
            harness
                .kv(&key)
                .is_some_and(|value| value.contains(&format!("status = \"{status}\"")))
        })
        .await;
    }

    async fn allocate(&self, name: &str, ports: u32) -> Result<Vec<Address>, Error> {
        let proposal = ServerProposal {
            name: name.to_string(),
            group: None,
            resources: resources(ports),
            specification: specification("echo"),
        };
        self.runtime()
            .call(async |bindings, store| {
                bindings
                    .atomic_plugin_system_node()
                    .call_allocate(store, NODE, &proposal)
                    .await
            })
            .await
            .unwrap()
    }

    async fn free(&self, addresses: &[Address]) {
        self.runtime()
            .call(async |bindings, store| {
                bindings
                    .atomic_plugin_system_node()
                    .call_free(store, NODE, addresses)
                    .await
            })
            .await
            .unwrap()
            .unwrap();
    }

    async fn start(&self, server: &Server) -> Result<Screen, Error> {
        let process = self
            .runtime()
            .call(async |bindings, store| {
                bindings
                    .atomic_plugin_system_node()
                    .call_start(store, NODE, server)
                    .await
            })
            .await
            .unwrap()?;
        Ok(Screen::new(self.runtime().context().clone(), process))
    }

    async fn restart(&self, server: &Server) {
        self.runtime()
            .call(async |bindings, store| {
                bindings
                    .atomic_plugin_system_node()
                    .call_restart(store, NODE, server)
                    .await
            })
            .await
            .unwrap()
            .unwrap();
    }

    /// Stops `server` the way the controller does: the guard is parked on
    /// the host and dropped once the plugin reports the server as stopped.
    async fn stop(&self, server: &Server) -> WeakGuard {
        let (guard, weak) = Guard::new();
        self.runtime()
            .context()
            .insert_guard(server.uuid.clone(), guard);
        self.runtime()
            .call(async |bindings, store| {
                bindings
                    .atomic_plugin_system_node()
                    .call_stop(store, NODE, server)
                    .await
            })
            .await
            .unwrap()
            .unwrap();
        weak
    }

    async fn wait_stopped(&self, guard: &WeakGuard) -> Duration {
        let start = Instant::now();
        self.tick_until("the server to stop", |_| guard.is_dropped())
            .await;
        start.elapsed()
    }

    fn server_record(&self, uuid: &str) -> Option<String> {
        self.kv(&format!("server/{NODE}/{uuid}"))
    }
}

fn write_template(data: &Path, template: &TemplateSpec<'_>) {
    let directory = data.join("templates").join(template.name);
    std::fs::create_dir_all(directory.join("prepare")).unwrap();
    std::fs::write(directory.join("prepare").join("note.txt"), "prepare only").unwrap();
    let shutdown = template
        .shutdown
        .map(|command| format!("shutdown = {command:?}\n"))
        .unwrap_or_default();
    let exclusions = template
        .exclusions
        .iter()
        .map(|exclusion| format!("{exclusion:?}"))
        .collect::<Vec<_>>()
        .join(", ");
    std::fs::write(
        directory.join("template.toml"),
        format!(
            r#"description = "test template"
version = "1.0.0"
authors = ["tests"]
exclusions = [{exclusions}]
{shutdown}
[environment]
GREETING = "hello"

[prepare.unix]
command = "sh"
args = ["-c", {prepare:?}]

[startup.unix]
command = "sh"
args = ["-c", {startup:?}]
"#,
            prepare = template.prepare,
            startup = template.startup,
        ),
    )
    .unwrap();
}

fn resources(ports: u32) -> Resources {
    Resources {
        memory: 512,
        swap: 0,
        cpu: 100,
        io: 500,
        disk: 1024,
        ports,
    }
}

fn specification(image: &str) -> Specification {
    Specification {
        settings: vec![],
        environment: vec![("EXTRA".to_string(), "1".to_string())],
        disk_retention: DiskRetention::Temporary,
        image: image.to_string(),
    }
}

fn server(uuid: &str, image: &str, port: u16) -> Server {
    Server {
        name: format!("server-{uuid}"),
        uuid: uuid.to_string(),
        group: None,
        allocation: Allocation {
            ports: vec![Address {
                host: "127.0.0.1".to_string(),
                port,
            }],
            resources: resources(1),
            specification: specification(image),
        },
        token: "sctl_test".to_string(),
        connected_users: 0,
    }
}

fn format_errors(errors: &[ScopedError]) -> String {
    errors
        .iter()
        .map(|error| format!("{}: {}", error.scope, error.message))
        .collect::<Vec<_>>()
        .join("; ")
}

/// Pulls the screen until a line equal to `needle` shows up.
async fn expect_line(screen: &Screen, needle: &str) {
    let start = Instant::now();
    let mut seen = vec![];
    loop {
        let Ok(Ok(lines)) = screen.pull().await else {
            panic!("pulling the screen failed");
        };
        seen.extend(lines);
        if seen.iter().any(|line| line == needle) {
            return;
        }
        assert!(
            start.elapsed() < WAIT,
            "never saw {needle:?} on the screen, got {seen:?}"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
async fn server_lifecycle() {
    let Some(harness) = Harness::new(&[TemplateSpec::default()]).await else {
        return;
    };
    harness.wait_template("echo", "ready").await;
    assert!(harness.data().join("templates/echo/prepared.txt").exists());

    let addresses = harness.allocate("server-a", 1).await.unwrap();
    assert_eq!(addresses.len(), 1);
    let port = addresses[0].port;
    assert!((40000..40010).contains(&port));

    let server = server("a", "echo", port);
    let screen = harness.start(&server).await.unwrap();
    assert!(screen.is_supported());
    expect_line(&screen, &format!("started on {port} with hello")).await;

    assert!(matches!(screen.write(b"ping\n").await, Ok(Ok(()))));
    expect_line(&screen, "got ping").await;

    let record = harness.server_record("a").unwrap();
    assert!(record.contains("state = \"running\""), "{record}");
    let directory = harness.data().join(
        record
            .lines()
            .find_map(|line| line.strip_prefix("directory = "))
            .unwrap()
            .trim_matches('"'),
    );
    assert!(directory.join("prepared.txt").exists());

    let guard = harness.stop(&server).await;
    let took = harness.wait_stopped(&guard).await;
    assert!(took < Duration::from_secs(2), "stop took {took:?}");
    assert!(harness.server_record("a").is_none());
    assert!(!directory.exists(), "temporary directory was not removed");
    assert!(
        harness
            .runtime()
            .context()
            .processes
            .list_ids("")
            .is_empty()
    );

    harness.free(&addresses).await;
    assert_eq!(harness.kv("ports").unwrap().trim(), "allocated = []");
}

#[tokio::test]
async fn restart_respawns_the_server() {
    let Some(harness) = Harness::new(&[TemplateSpec::default()]).await else {
        return;
    };
    harness.wait_template("echo", "ready").await;
    let server = server("r", "echo", 40001);
    let screen = harness.start(&server).await.unwrap();
    expect_line(&screen, "started on 40001 with hello").await;

    harness.restart(&server).await;
    expect_line(&screen, "bye").await;
    harness
        .tick_until("the server to run again", |harness| {
            harness
                .server_record("r")
                .is_some_and(|record| record.contains("state = \"running\""))
        })
        .await;
    expect_line(&screen, "started on 40001 with hello").await;

    let guard = harness.stop(&server).await;
    harness.wait_stopped(&guard).await;
}

#[tokio::test]
async fn unresponsive_server_is_killed_after_the_stop_timeout() {
    let stubborn = TemplateSpec {
        name: "stubborn",
        startup: "trap '' TERM; echo stubborn; while :; do sleep 0.1; done",
        ..TemplateSpec::default()
    };
    let Some(harness) = Harness::new(&[stubborn]).await else {
        return;
    };
    harness.wait_template("stubborn", "ready").await;
    let server = server("s", "stubborn", 40002);
    let screen = harness.start(&server).await.unwrap();
    expect_line(&screen, "stubborn").await;

    let guard = harness.stop(&server).await;
    let took = harness.wait_stopped(&guard).await;
    assert!(took >= Duration::from_secs(2), "stopped after {took:?}");
    assert!(harness.server_record("s").is_none());
}

#[tokio::test]
async fn starting_from_an_unknown_template_fails() {
    let Some(harness) = Harness::new(&[TemplateSpec::default()]).await else {
        return;
    };
    let result = harness.start(&server("x", "missing", 40003)).await;
    assert!(matches!(result, Err(Error::NotFound(_))));
}

#[tokio::test]
async fn stopping_an_unknown_server_completes_immediately() {
    let Some(harness) = Harness::new(&[TemplateSpec::default()]).await else {
        return;
    };
    let guard = harness.stop(&server("gone", "echo", 40004)).await;
    assert!(guard.is_dropped());
}

#[tokio::test]
async fn directory_exclusions_are_not_copied() {
    let template = TemplateSpec {
        exclusions: &["template.toml", "prepare/"],
        ..TemplateSpec::default()
    };
    let Some(harness) = Harness::new(&[template]).await else {
        return;
    };
    harness.wait_template("echo", "ready").await;
    let server = server("e", "echo", 40006);
    let screen = harness.start(&server).await.unwrap();
    expect_line(&screen, "started on 40006 with hello").await;

    let directory = harness.data().join("servers/temporary").join(NODE);
    let server_directory = std::fs::read_dir(&directory)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert!(server_directory.join("prepared.txt").exists());
    assert!(!server_directory.join("template.toml").exists());
    assert!(
        !server_directory.join("prepare").exists(),
        "excluded directory was copied"
    );

    let guard = harness.stop(&server).await;
    harness.wait_stopped(&guard).await;
}

#[tokio::test]
async fn mapped_ports_are_reserved_and_claimed() {
    let Some(harness) = Harness::new(&[TemplateSpec::default()]).await else {
        return;
    };
    let ports = |addresses: &[Address]| addresses.iter().map(|a| a.port).collect::<Vec<_>>();

    // Take the first five ports of the range
    let first = harness.allocate("x", 5).await.unwrap();
    assert_eq!(ports(&first), vec![40000, 40001, 40002, 40003, 40004]);
    // 40005 is mapped to "lobby", so other servers skip it
    let other = harness.allocate("y", 1).await.unwrap();
    assert_eq!(ports(&other), vec![40006]);

    let lobby = harness.allocate("lobby", 1).await.unwrap();
    assert_eq!(ports(&lobby), vec![40005]);
    // A second lobby cannot get the same port while the first holds it
    assert!(matches!(
        harness.allocate("lobby", 1).await,
        Err(Error::AlreadyExists(_))
    ));
    harness.free(&lobby).await;
    assert_eq!(
        ports(&harness.allocate("lobby", 1).await.unwrap()),
        vec![40005]
    );
}

#[tokio::test]
async fn concurrent_allocations_never_share_ports() {
    let Some(harness) = Harness::new(&[TemplateSpec::default()]).await else {
        return;
    };
    let harness = Arc::new(harness);
    let mut tasks = vec![];
    for index in 0..8 {
        let harness = harness.clone();
        tasks.push(tokio::spawn(async move {
            harness.allocate(&format!("s{index}"), 1).await.unwrap()[0].port
        }));
    }
    let mut ports = vec![];
    for task in tasks {
        ports.push(task.await.unwrap());
    }
    ports.sort_unstable();
    ports.dedup();
    assert_eq!(ports.len(), 8, "a port was handed out twice: {ports:?}");
    assert!(!ports.contains(&40005));
}

#[tokio::test]
async fn servers_without_a_shutdown_command_get_terminated() {
    let template = TemplateSpec {
        name: "signals",
        shutdown: None,
        startup: "trap 'echo terminated; exit 0' TERM; echo waiting; while :; do sleep 0.1; done",
        ..TemplateSpec::default()
    };
    let Some(harness) = Harness::new(&[template]).await else {
        return;
    };
    harness.wait_template("signals", "ready").await;
    let server = server("t", "signals", 40007);
    let screen = harness.start(&server).await.unwrap();
    expect_line(&screen, "waiting").await;

    let guard = harness.stop(&server).await;
    expect_line(&screen, "terminated").await;
    let took = harness.wait_stopped(&guard).await;
    // The stop timeout is 2s; a graceful exit must not have to wait for it
    assert!(took < Duration::from_millis(1500), "stop took {took:?}");
}

#[tokio::test]
async fn crashed_servers_are_cleaned_up() {
    let template = TemplateSpec {
        name: "crashing",
        startup: "echo crashing; sleep 0.2; exit 1",
        ..TemplateSpec::default()
    };
    let Some(harness) = Harness::new(&[template]).await else {
        return;
    };
    harness.wait_template("crashing", "ready").await;
    let server = server("c", "crashing", 40008);
    harness.start(&server).await.unwrap();
    assert!(harness.server_record("c").is_some());

    harness
        .tick_until("the crashed server to be cleaned up", |harness| {
            harness.server_record("c").is_none()
        })
        .await;
    assert!(
        harness
            .runtime()
            .context()
            .processes
            .list_ids("server/")
            .is_empty()
    );
    assert!(
        std::fs::read_dir(harness.data().join("servers/temporary").join(NODE))
            .unwrap()
            .next()
            .is_none(),
        "temporary directory of the crashed server was kept"
    );

    // The controller stops it later; that must complete right away
    let guard = harness.stop(&server).await;
    assert!(guard.is_dropped());
}

#[tokio::test]
async fn templates_whose_prepare_fails_are_not_used() {
    let template = TemplateSpec {
        name: "broken",
        prepare: "echo downloading; exit 3",
        ..TemplateSpec::default()
    };
    let Some(harness) = Harness::new(&[template]).await else {
        return;
    };
    harness.wait_template("broken", "failed").await;
    assert!(harness.kv("template/broken").unwrap().contains("Code(3)"));
    let result = harness.start(&server("b", "broken", 40009)).await;
    assert!(
        matches!(&result, Err(Error::NotReady(message)) if message.contains("failed to prepare")),
        "unexpected result: {:?}",
        result.err()
    );
}
