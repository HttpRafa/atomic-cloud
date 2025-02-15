use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use anyhow::Result;
use auth::manager::AuthManager;
use getset::{Getters, MutGetters};
use group::manager::GroupManager;
use node::manager::NodeManager;
use plugin::manager::PluginManager;
use server::{manager::ServerManager, screen::manager::ScreenManager};
use simplelog::info;
use subscriber::manager::SubscriberManager;
use tokio::{
    select,
    sync::mpsc::{channel, Receiver, Sender},
    time::{interval, MissedTickBehavior},
};
use user::manager::UserManager;

use crate::{config::Config, network::NetworkStack, task::Task};

pub mod auth;
pub mod group;
pub mod node;
pub mod plugin;
pub mod server;
pub mod subscriber;
pub mod user;

const TICK_RATE: u64 = 10;
const TASK_BUFFER: usize = 128;

pub type TaskSender = Sender<Task>;

#[derive(Getters, MutGetters)]
pub struct Controller {
    /* State */
    running: Arc<AtomicBool>,

    /* Tasks */
    tasks: (TaskSender, Receiver<Task>),

    /* Shared Components */
    pub shared: Arc<Shared>,

    /* Components */
    pub plugins: PluginManager,
    pub nodes: NodeManager,
    pub groups: GroupManager,
    pub servers: ServerManager,
    pub users: UserManager,

    /* Config */
    #[getset(get = "pub")]
    config: Config,
}

pub struct Shared {
    pub auth: AuthManager,
    pub subscribers: SubscriberManager,
    pub screens: ScreenManager,
}

impl Controller {
    pub async fn init(config: Config) -> Result<Self> {
        let shared = Arc::new(Shared {
            auth: AuthManager::init().await?,
            subscribers: SubscriberManager::init(),
            screens: ScreenManager::init(),
        });

        let plugins = PluginManager::init(&config).await?;
        let nodes = NodeManager::init(&plugins).await?;
        let groups = GroupManager::init(&nodes).await?;

        let servers = ServerManager::init();
        let users = UserManager::init();

        Ok(Self {
            running: Arc::new(AtomicBool::new(true)),
            tasks: channel(TASK_BUFFER),
            shared,
            plugins,
            nodes,
            groups,
            servers,
            users,
            config,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Setup signal handlers
        self.setup_handlers()?;

        let network = NetworkStack::start(&self.config, &self.shared, &self.tasks.0);

        // Main loop
        let mut interval = interval(Duration::from_millis(1000 / TICK_RATE));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        while self.running.load(Ordering::Relaxed) {
            select! {
                _ = interval.tick() => self.tick().await?,
                task = self.tasks.1.recv() => if let Some(task) = task {
                    task.run(self).await?;
                }
            }
        }

        // Shutdown
        self.shutdown(network).await?;

        Ok(())
    }

    async fn tick(&mut self) -> Result<()> {
        // Tick plugin manager
        self.plugins.tick()?;

        // Tick node manager
        self.nodes.tick()?;

        // Tick group manager
        self.groups.tick(&self.config, &mut self.servers)?;

        // Tick server manager
        self.servers
            .tick(
                &self.config,
                &self.nodes,
                &mut self.groups,
                &mut self.users,
                &self.shared,
            )
            .await?;

        // Tick user manager
        self.users.tick(&self.config)?;

        // Tick subscriber manager
        self.shared.subscribers.tick().await?;

        // Tick screen manager
        self.shared.screens.tick().await?;

        Ok(())
    }

    async fn shutdown(&mut self, network: NetworkStack) -> Result<()> {
        info!("Starting shutdown sequence...");

        // Shutdown user manager
        self.users.shutdown()?;

        // Shutdown screen manager
        self.shared.screens.shutdown().await?;

        // Shutdown server manager
        self.servers.shutdown()?;

        // Shutdown group manager
        self.groups.shutdown()?;

        // Shutdown node manager
        self.nodes.shutdown().await?;

        // Shutdown plugin manager
        self.plugins.shutdown().await?;

        // Shutdown network stack
        network.shutdown().await?;

        info!("Shutdown complete. Bye :)");
        Ok(())
    }

    fn setup_handlers(&self) -> Result<()> {
        let flag = self.running.clone();
        ctrlc::set_handler(move || {
            info!("Received SIGINT, shutting down...");
            flag.store(false, Ordering::Relaxed);
        })
        .map_err(std::convert::Into::into)
    }

    pub fn signal_shutdown(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}
