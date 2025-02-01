use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use anyhow::Result;
use plugin::manager::PluginManager;
use simplelog::info;
use tokio::{
    select,
    sync::mpsc::{channel, Receiver, Sender},
    time::interval,
};

use crate::{
    config::Config,
    task::{Task, WrappedTask},
};

mod plugin;

const TICK_RATE: u64 = 20;
const TASK_BUFFER: usize = 128;

pub type TaskSender = Sender<WrappedTask>;

pub struct Controller {
    /* State */
    running: Arc<AtomicBool>,

    /* Tasks */
    tasks: (TaskSender, Receiver<WrappedTask>),

    /* Components */
    plugin: PluginManager,

    /* Config */
    config: Config,
}

impl Controller {
    pub async fn init(config: Config) -> Result<Self> {
        Ok(Self {
            running: Arc::new(AtomicBool::new(true)),
            tasks: channel(TASK_BUFFER),
            plugin: PluginManager::init(&config).await?,
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
        self.plugin.tick().await?;

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Starting shutdown sequence...");

        // Shutdown plugin manager
        self.plugin.shutdown().await?;

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
