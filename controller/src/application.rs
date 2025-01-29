use anyhow::Error;
use auth::Auth;
use cloudlet::Cloudlets;
use deployment::Deployments;
use driver::Drivers;
use event::EventBus;
use simplelog::info;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::{Builder, Runtime};
use unit::Units;
use user::Users;

use crate::config::Config;
use crate::network::NetworkStack;

pub mod auth;
pub mod cloudlet;
pub mod deployment;
pub mod driver;
pub mod event;
pub mod unit;
pub mod user;

static STARTUP_SLEEP: Duration = Duration::from_secs(1);
static SHUTDOWN_WAIT: Duration = Duration::from_secs(10);

const TICK_RATE: u64 = 1;

pub struct Controller {
    /* Runtime State */
    running: Arc<AtomicBool>,

    /* Configuration */
    configuration: Config,

    /* Drivers */
    drivers: Drivers,

    /* Authentication */
    auth: Auth,
    users: Users,

    /* Resources */
    cloudlets: Cloudlets,
    deployments: Deployments,
    units: Units,

    /* Event Bus */
    event_bus: EventBus,
}

impl Controller {
    pub async fn new(configuration: Config) -> Self {
        let auth = Auth::load_all().await;
        let drivers = Drivers::load_all(&configuration.identifier).await;
        let cloudlets = Cloudlets::load_all(handle.clone(), &drivers);
        let deployments = Deployments::load_all(handle.clone(), &cloudlets);
        let units = Units::new(handle.clone());
        let users = Users::new(handle.clone());
        let event_bus = EventBus::new(/*handle.clone()*/);
        Self {
            configuration,
            drivers,
            runtime: RwLock::new(Some(
                Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create Tokio runtime"),
            )),
            running: Arc::new(AtomicBool::new(true)),
            auth,
            cloudlets: RwLock::new(cloudlets),
            deployments: RwLock::new(deployments),
            units,
            users,
            event_bus,
        }
    }

    pub async fn start(&self) {
        // Set up signal handlers
        self.setup_interrupts();

        let network_handle = NetworkStack::start(self.handle.upgrade().unwrap());
        let tick_duration = Duration::from_millis(1000 / TICK_RATE);

        // Wait for 1 second before starting the tick loop
        thread::sleep(STARTUP_SLEEP);

        while self.running.load(Ordering::Relaxed) {
            let start_time = Instant::now();
            self.tick();

            let elapsed_time = start_time.elapsed();
            if elapsed_time < tick_duration {
                thread::sleep(tick_duration - elapsed_time);
            }
        }

        // Stop all units
        info!("<red>Stopping</> all units...");
        self.units.stop_all_instant();

        // Stop network stack
        info!("<red>Stopping</> network stack...");
        network_handle.shutdown();

        // Let the drivers cleanup there messes
        info!("Letting the drivers <red>cleanup</>...");
        self.drivers.cleanup();
    }

    pub fn request_stop(&self) {
        info!("Controller <red>stop</> requested. <red>Stopping</>...");
        self.running.store(false, Ordering::Relaxed);
    }

    pub fn get_config(&self) -> &Config {
        &self.configuration
    }

    pub fn get_drivers(&self) -> &Drivers {
        &self.drivers
    }

    pub fn get_auth(&self) -> &Auth {
        &self.auth
    }

    pub fn get_units(&self) -> &Units {
        &self.units
    }

    pub fn get_users(&self) -> &Users {
        &self.users
    }

    pub fn get_event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    pub fn get_runtime(&self) -> RwLockReadGuard<Option<Runtime>> {
        self.runtime.read().expect("Failed to get lock to runtime")
    }

    fn tick(&self) {
        // Tick all drivers
        self.drivers.tick();

        // Tick all driver cloudlets
        self.lock_cloudlets().tick();

        // Check if all deployments have started there units etc..
        self.lock_deployments().tick(&self.units);

        // Check if all units have sent their heartbeats and start requested units if we can
        self.units.tick();

        // Check state of all users
        self.users.tick();
    }

    fn setup_interrupts(&self) {
        // Set up signal handlers
        let running = self.running.clone();
        ctrlc::set_handler(move || {
            info!("<red>Interrupt</> signal received. Stopping...");
            running.store(false, Ordering::Relaxed);
        }).expect("Failed to set Ctrl+C handler");
    }
}

pub enum CreationResult {
    Created,
    AlreadyExists,
    Denied(Error),
}
