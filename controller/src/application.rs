use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use anyhow::Result;
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

use crate::{
    config::Config,
    task::WrappedTask,
};

mod plugin;
mod node;
mod group;
mod server;

const TICK_RATE: u64 = 20;
const TASK_BUFFER: usize = 128;

pub type TaskSender = Sender<WrappedTask>;

pub struct Controller {
    /* State */
    running: Arc<AtomicBool>,

    /* Tasks */
    tasks: (TaskSender, Receiver<WrappedTask>),

    /* Components */
    plugins: PluginManager,
    nodes: NodeManager,
    groups: GroupManager,
    servers: ServerManager,

    /* Config */
    config: Config,
}

impl Controller {
    pub async fn init(config: Config) -> Result<Self> {
        Ok(Self {
            running: Arc::new(AtomicBool::new(true)),
            tasks: channel(TASK_BUFFER),
            plugins: PluginManager::init(&config).await?,
            nodes: NodeManager::init().await?,
            groups: GroupManager::init().await?,
            servers: ServerManager::init().await?,
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
        self.groups.tick().await?;

        // Tick server manager
        self.servers.tick().await?;

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

pub trait TickService {
    async fn tick(&mut self) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;
}
