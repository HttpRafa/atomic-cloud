use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use anyhow::Result;
use auth::validator::{AuthValidator, WrappedAuthValidator};
use getset::{Getters, MutGetters};
use group::manager::GroupManager;
use node::manager::NodeManager;
use plugin::manager::PluginManager;
use server::manager::ServerManager;
use simplelog::info;
use tokio::{
    select,
    sync::mpsc::{channel, Receiver, Sender},
    time::interval,
};

use crate::{config::Config, task::WrappedTask};

mod auth;
mod group;
mod node;
mod plugin;
mod server;

const TICK_RATE: u64 = 20;
const TASK_BUFFER: usize = 128;

pub type TaskSender = Sender<WrappedTask>;

#[derive(Getters, MutGetters)]
pub struct Controller {
    /* State */
    running: Arc<AtomicBool>,

    /* Tasks */
    tasks: (TaskSender, Receiver<WrappedTask>),

    /* Auth */
    validator: WrappedAuthValidator,

    /* Components */
    #[getset(get = "pub", get_mut = "pub")]
    plugins: PluginManager,
    #[getset(get = "pub", get_mut = "pub")]
    nodes: NodeManager,
    #[getset(get = "pub", get_mut = "pub")]
    groups: GroupManager,
    #[getset(get = "pub", get_mut = "pub")]
    servers: ServerManager,

    /* Config */
    #[getset(get = "pub")]
    config: Config,
}

impl Controller {
    pub async fn init(config: Config) -> Result<Self> {
        let validator = AuthValidator::init()?;

        let plugins = PluginManager::init(&config).await?;
        let nodes = NodeManager::init(&plugins).await?;
        let groups = GroupManager::init(&nodes).await?;
        let servers = ServerManager::init().await?;

        Ok(Self {
            running: Arc::new(AtomicBool::new(true)),
            tasks: channel(TASK_BUFFER),
            validator,
            plugins,
            nodes,
            groups,
            servers,
            config,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Setup signal handlers
        self.setup_handlers()?;

        // Main loop
        let mut interval = interval(Duration::from_millis(1000 / TICK_RATE));
        while self.running.load(Ordering::Relaxed) {
            select! {
                _ = interval.tick() => self.tick().await?,
                task = self.tasks.1.recv() => if let Some(mut task) = task {
                    task.run(self).await?;
                }
            }
        }

        // Shutdown
        self.shutdown().await?;

        Ok(())
    }

    async fn tick(&mut self) -> Result<()> {
        // Tick plugin manager
        self.plugins.tick().await?;

        // Tick node manager
        self.nodes.tick().await?;

        // Tick group manager
        self.groups.tick(&self.config, &mut self.servers).await?;

        // Tick server manager
        self.servers
            .tick(&self.config, &self.nodes, &mut self.groups, &self.validator)
            .await?;

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Starting shutdown sequence...");

        // Shutdown server manager
        self.servers.shutdown().await?;

        // Shutdown group manager
        self.groups.shutdown().await?;

        // Shutdown node manager
        self.nodes.shutdown().await?;

        // Shutdown plugin manager
        self.plugins.shutdown().await?;

        info!("Shutdown complete. Bye :)");
        Ok(())
    }

    fn setup_handlers(&self) -> Result<()> {
        let flag = self.running.clone();
        ctrlc::set_handler(move || {
            info!("Received SIGINT, shutting down...");
            flag.store(false, Ordering::Relaxed);
        })
        .map_err(|error| error.into())
    }
}
