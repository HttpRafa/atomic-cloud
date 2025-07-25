use std::{
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
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
use simplelog::{error, info};
use subscriber::manager::SubscriberManager;
use tls::TlsSetting;
use tokio::{
    select,
    sync::watch,
    time::{Instant, MissedTickBehavior, interval},
};
use user::manager::UserManager;

use crate::{config::Config, network::NetworkStack, task::manager::TaskManager};

pub mod auth;
pub mod group;
pub mod node;
pub mod plugin;
pub mod server;
pub mod subscriber;
pub mod tls;
pub mod user;

pub const TICK_RATE: u64 = 10;

#[derive(Getters, MutGetters)]
pub struct Controller {
    /* State */
    state: State,

    /* Tasks */
    tasks: TaskManager,

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

// This is data that is shared between network thread/tasks and plugin execution threads/tasks
pub struct Shared {
    pub auth: AuthManager,
    pub subscribers: SubscriberManager,
    pub screens: ScreenManager,
    pub tls: TlsSetting,
}

impl Controller {
    pub async fn init(config: Config) -> Result<Self> {
        let shared = Arc::new(Shared {
            auth: AuthManager::init().await?,
            subscribers: SubscriberManager::init(),
            screens: ScreenManager::init(),
            tls: TlsSetting::init(&config).await?,
        });

        let tasks = TaskManager::init();

        let plugins = PluginManager::init(&config, &tasks.get_sender(), &shared).await?;
        let nodes = NodeManager::init(&plugins).await?;
        let groups = GroupManager::init(&nodes).await?;

        let servers = ServerManager::init();
        let users = UserManager::init();

        Ok(Self {
            state: State::new(),
            tasks,
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
        // Set task system to ready
        self.tasks.set_ready(true);

        // Setup signal handlers
        self.setup_handlers()?;

        let network =
            NetworkStack::start(&self.config, self.shared.clone(), self.tasks.get_sender());

        // Main loop
        let mut interval = interval(Duration::from_millis(1000 / TICK_RATE));
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
        while self.state.running {
            self.state.tick(); // Check for exit votes

            select! {
                _ = interval.tick() => self.tick(&network).await?,
                task = self.tasks.recv() => if let Some(task) = task {
                    task.run(self).await?;
                },
                _ = self.state.signal.1.changed() => self.shutdown()?,
            }
        }

        // Set task system to not ready
        self.tasks.set_ready(false);

        // Cleanup
        self.cleanup(network).await?;

        info!("Shutdown complete. Bye :)");
        Ok(())
    }

    async fn tick(&mut self, _network: &NetworkStack) -> Result<()> {
        let start = Instant::now();

        // Tick plugin manager
        self.plugins.tick().await?;

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

        // Check if tick took longer than expected
        let elapsed = start.elapsed();
        #[allow(clippy::cast_possible_truncation)]
        if elapsed.as_millis() as u64 > 1000 / TICK_RATE {
            info!("Tick took longer than expected: {:?}", elapsed);
        }

        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        info!("Starting shutdown sequence...");

        // Shutdown group manager
        self.groups.shutdown(self.state.vote())?;

        // Shutdown server manager
        self.servers.shutdown(self.state.vote())?;
        Ok(())
    }

    async fn cleanup(&mut self, network: NetworkStack) -> Result<()> {
        info!("Starting cleanup sequence...");

        // Cleanup user manager
        self.users.cleanup()?;

        // Cleanup server manager
        self.servers.cleanup()?;

        // Cleanup group manager
        self.groups.cleanup()?;

        // Cleanup node manager
        self.nodes.cleanup().await?;

        // Cleanup subscription manager
        self.shared.subscribers.cleanup().await?;

        // Cleanup screen manager
        self.shared.screens.cleanup().await?;

        // Cleanup plugin manager
        self.plugins.cleanup().await?;

        // Shutdown network stack
        network.shutdown().await?;
        Ok(())
    }

    fn setup_handlers(&self) -> Result<()> {
        let sender = self.state.signal.0.clone();
        ctrlc::set_handler(move || {
            info!("Received SIGINT, shutting down...");
            if let Err(error) = sender.send(false) {
                error!("Failed to send shutdown signal: {}", error);
            }
        })
        .map_err(std::convert::Into::into)
    }

    pub fn signal_shutdown(&self) {
        if let Err(error) = self.state.signal.0.send(false) {
            error!("Failed to send shutdown signal: {}", error);
        }
    }
}

struct State {
    pub running: bool,
    pub signal: (watch::Sender<bool>, watch::Receiver<bool>),
    pub votes: (bool, u8, Arc<AtomicU8>),
}

pub struct Voter(bool, Arc<AtomicU8>);
pub type OptVoter = Option<Voter>;

impl Voter {
    pub fn vote(&mut self) -> bool {
        if self.0 {
            self.1.fetch_add(1, Ordering::Relaxed);
            self.0 = false;
            true
        } else {
            false
        }
    }
}

impl State {
    #[must_use]
    fn new() -> Self {
        Self {
            running: true,
            signal: watch::channel(true),
            votes: (false, 0, Arc::new(AtomicU8::new(0))),
        }
    }

    fn tick(&mut self) {
        if self.votes.0 && self.votes.1 <= self.votes.2.load(Ordering::Relaxed) {
            info!("Received enough votes to exit, initiating...");
            self.running = false;
        }
    }

    fn vote(&mut self) -> Voter {
        self.votes.0 = true;
        self.votes.1 += 1;
        Voter(true, self.votes.2.clone())
    }
}
