use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc, RwLock, RwLockReadGuard, Weak,
    },
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use simplelog::{debug, info, warn, error};
use uuid::Uuid;

use super::{
    auth::AuthUnitHandle,
    cloudlet::{AllocationHandle, CloudletHandle, WeakCloudletHandle},
    deployment::WeakDeploymentHandle,
    ControllerHandle, WeakControllerHandle,
};

pub type UnitHandle = Arc<Unit>;
pub type WeakUnitHandle = Weak<Unit>;
pub type StartRequestHandle = Arc<StartRequest>;

pub struct Units {
    controller: WeakControllerHandle,

    /* Units started by this atomic cloud instance */
    units: RwLock<HashMap<Uuid, UnitHandle>>,

    /* Units that should be started/stopped next controller tick */
    start_requests: RwLock<VecDeque<StartRequestHandle>>,
    stop_requests: RwLock<VecDeque<StopRequest>>,
}

impl Units {
    pub fn new(controller: WeakControllerHandle) -> Self {
        Self {
            controller,
            units: RwLock::new(HashMap::new()),
            start_requests: RwLock::new(VecDeque::new()),
            stop_requests: RwLock::new(VecDeque::new()),
        }
    }

    pub fn tick(&self) {
        // Get Controller handle
        let controller = self
            .controller
            .upgrade()
            .expect("Failed to upgrade controller");

        // Check health of units
        {
            let dead_units = self.units.read().unwrap().values().filter(|unit| {
                let health = unit.health.read().unwrap();
                if health.is_dead() {
                    match *unit.state.read().unwrap() {
                        State::Starting | State::Restarting => {
                            warn!("Unit <blue>{} <red>failed</> to establish online status within the expected startup time of <blue>{:.2?}</>.", unit.name, controller.configuration.timings.restart.unwrap());
                        }
                        _ => {
                            warn!("Unit <blue>{}</> has not checked in for <blue>{:.2?}</>, indicating a potential error.", unit.name, health.timeout);
                        }
                    }
                    true
                } else {
                    false
                }
            }).cloned().collect::<Vec<_>>();
            for unit in dead_units {
                self.restart_unit(&unit);
            }
        }

        // Stop all units that have to be stopped
        {
            let mut requests = self.stop_requests.write().unwrap();
            requests.retain(|request| {
                if let Some(when) = request.when {
                    if when > Instant::now() {
                        return true;
                    }
                }

                self.stop_unit_nolock(request, &mut self.units.write().unwrap());
                false
            });
        }

        // Sort requests by priority and process them
        {
            let mut requests = self.start_requests.write().unwrap();
            {
                let contiguous = requests.make_contiguous();
                contiguous.sort_unstable_by_key(|req| req.priority);
                contiguous.reverse();
            }
            requests.retain(|request| {
                if request.canceled.load(Ordering::Relaxed) {
                    debug!(
                        "<yellow>Canceled</> start of unit <blue>{}</>",
                        request.name
                    );
                    return false;
                }

                if let Some(when) = request.when {
                    if when > Instant::now() {
                        return true;
                    }
                }

                if request.cloudlets.is_empty() {
                    warn!(
                        "<red>Failed</> to allocate resources for unit <red>{}</> because no cloudlets were specified",
                        request.name
                    );
                    return true;
                }

                // Collect and sort cloudlets by the number of allocations
                for cloudlet in &request.cloudlets {
                    let cloudlet = cloudlet.upgrade().unwrap();
                    // Try to allocate resources on cloudlets
                    if let Ok(allocation) = cloudlet.allocate(request) {
                        // Start unit on the cloudlet
                        self.start_unit(request, allocation, &cloudlet);
                        return false;
                    }
                }
                warn!(
                    "<red>Failed</> to allocate resources for unit <red>{}</>",
                    request.name
                );
                true
            });
        }
    }

    pub fn queue_unit(&self, request: StartRequest) -> StartRequestHandle {
        let arc = Arc::new(request);
        self.start_requests.write().unwrap().push_back(arc.clone());
        arc
    }

    pub fn stop_all_instant(&self) {
        self.units.write().unwrap().drain().for_each(|(_, unit)| {
            self.stop_unit_internal(&StopRequest { when: None, unit });
        });
    }

    pub fn stop_all_on_cloudlet(&self, cloudlet: &CloudletHandle) {
        self.units
            .read()
            .unwrap()
            .values()
            .filter(|unit| Arc::ptr_eq(&unit.cloudlet.upgrade().unwrap(), cloudlet))
            .for_each(|unit| {
                self.stop_unit_now(unit.clone());
            });
    }

    fn stop_unit_nolock(&self, request: &StopRequest, units: &mut HashMap<Uuid, UnitHandle>) {
        self.stop_unit_internal(request);
        units.remove(&request.unit.uuid);
    }

    fn stop_unit_internal(&self, request: &StopRequest) {
        let unit = &request.unit;
        info!("<red>Stopping</> unit <blue>{}</>", unit.name);

        // Remove resources allocated by unit from cloudlet
        if let Some(cloudlet) = unit.cloudlet.upgrade() {
            cloudlet.deallocate(&unit.allocation);
        }

        // Send start request to cloudlet
        // We do this async because the driver chould be running blocking code like network requests
        let controller = self
            .controller
            .upgrade()
            .expect("The controller is dead while still running code that requires it");
        {
            let unit = unit.clone();
            controller
                .get_runtime()
                .as_ref()
                .unwrap()
                .spawn_blocking(move || stop_thread(unit));
        }

        // Remove unit from deployment and units list
        if let Some(deployment) = &unit.deployment {
            deployment.remove_unit(unit);
        }
        if let Some(controller) = self.controller.upgrade() {
            controller.get_auth().unregister_unit(unit);
        }

        // Remove users connected to the unit
        controller.get_users().cleanup_users(unit);
        // Remove subscribers from channels
        controller.get_event_bus().cleanup_unit(unit);

        fn stop_thread(unit: UnitHandle) {
            if let Some(cloudlet) = unit.cloudlet.upgrade() {
                if let Err(error) = cloudlet.get_inner().stop_unit(&unit) {
                    error!(
                        "<red>Failed</> to stop unit <red>{}</>: <red>{}</>",
                        unit.name,
                        error
                    );
                }
            }
        }
    }

    pub fn stop_unit_now(&self, unit: UnitHandle) {
        self.stop_requests
            .write()
            .unwrap()
            .push_back(StopRequest { when: None, unit });
    }

    pub fn _stop_unit(&self, when: Instant, unit: UnitHandle) {
        self.stop_requests.write().unwrap().push_back(StopRequest {
            when: Some(when),
            unit,
        });
    }

    pub fn restart_unit(&self, unit: &UnitHandle) {
        info!("<yellow>Restarting</> unit <blue>{}</>", unit.name);

        let controller = self
            .controller
            .upgrade()
            .expect("Failed to upgrade controller");

        *unit.state.write().unwrap() = State::Restarting;
        *unit.health.write().unwrap() = Health::new(
            controller.configuration.timings.restart.unwrap(),
            controller.configuration.timings.healthbeat.unwrap(),
        );

        // Send restart request to cloudlet
        // We do this async because the driver chould be running blocking code like network requests
        if let Some(controller) = self.controller.upgrade() {
            let unit = unit.clone();
            let copy = controller.clone();
            controller
                .get_runtime()
                .as_ref()
                .unwrap()
                .spawn_blocking(move || restart_thread(copy, unit));
        }

        fn restart_thread(controller: ControllerHandle, unit: UnitHandle) {
            if let Some(cloudlet) = unit.cloudlet.upgrade() {
                if let Err(error) = &cloudlet.get_inner().restart_unit(&unit) {
                    error!(
                        "<red>Failed</> to restart unit <red>{}</>: <red>{}</>",
                        unit.name,
                        error
                    );
                    controller.get_units().stop_unit_now(unit);
                }
            }
        }
    }

    pub fn handle_heart_beat(&self, unit: &UnitHandle) {
        debug!("Received heartbeat from unit {}", &unit.name);

        // Reset health
        unit.health.write().unwrap().reset();

        // Check were the unit is in the state machine
        let mut state = unit.state.write().unwrap();
        if *state == State::Starting || *state == State::Restarting {
            *state = State::Preparing;
            info!(
                "The unit <blue>{}</> is now <yellow>loading</>",
                unit.name
            );
        }
    }

    pub fn mark_ready(&self, unit: &UnitHandle) {
        if !unit.rediness.load(Ordering::Relaxed) {
            debug!("The unit <blue>{}</> is <green>ready</>", unit.name);
            unit.rediness.store(true, Ordering::Relaxed);
        }
    }

    pub fn mark_not_ready(&self, unit: &UnitHandle) {
        if unit.rediness.load(Ordering::Relaxed) {
            debug!(
                "The unit <blue>{}</> is <red>no longer</> ready",
                unit.name
            );
            unit.rediness.store(false, Ordering::Relaxed);
        }
    }

    pub fn mark_running(&self, unit: &UnitHandle) {
        let mut state = unit.state.write().unwrap();
        if *state == State::Preparing {
            info!("The unit <blue>{}</> is now <green>running</>", unit.name);
            *state = State::Running;
        }
    }

    pub fn checked_unit_stop(&self, unit: &UnitHandle) {
        let mut state = unit.state.write().unwrap();
        if *state != State::Stopping {
            self.mark_not_ready(unit);
            *state = State::Stopping;
            self.stop_unit_now(unit.clone());
        }
    }

    pub fn find_fallback_unit(&self, excluded: &UnitHandle) -> Option<UnitHandle> {
        // TODO: Also check if the unit have free slots
        self.units
            .read()
            .unwrap()
            .values()
            .filter(|unit| {
                !Arc::ptr_eq(unit, excluded)
                    && unit.allocation.spec.fallback.enabled
                    && *unit.state.read().unwrap() == State::Running
            })
            .max_by_key(|unit| unit.allocation.spec.fallback.priority)
            .cloned()
    }

    pub fn get_unit(&self, uuid: Uuid) -> Option<UnitHandle> {
        self.units.read().unwrap().get(&uuid).cloned()
    }

    pub fn get_units(&self) -> RwLockReadGuard<HashMap<Uuid, UnitHandle>> {
        self.units.read().expect("Failed to lock units")
    }

    fn start_unit(
        &self,
        request: &StartRequestHandle,
        allocation: AllocationHandle,
        cloudlet: &CloudletHandle,
    ) {
        let controller = self
            .controller
            .upgrade()
            .expect("Failed to upgrade controller");

        info!(
            "<green>Spinning up</> unit <blue>{}</> on cloudlet <blue>{}</> listening on port <blue>{}</>",
            request.name,
            cloudlet.name,
            allocation.primary_address().to_string()
        );
        let unit = Arc::new_cyclic(|handle| {
            // Create a token for the unit
            let auth = self
                .controller
                .upgrade()
                .expect("WAIT. We are creating a unit while the controller is dead?")
                .get_auth()
                .register_unit(handle.clone());

            Unit {
                name: request.name.clone(),
                uuid: Uuid::new_v4(),
                deployment: request.deployment.clone(),
                cloudlet: Arc::downgrade(cloudlet),
                allocation,
                connected_users: AtomicU32::new(0),
                auth,
                health: RwLock::new(Health::new(
                    controller.configuration.timings.startup.unwrap(),
                    controller.configuration.timings.healthbeat.unwrap(),
                )),
                state: RwLock::new(State::Starting),
                rediness: AtomicBool::new(false),
                flags: Flags {
                    stop: RwLock::new(None),
                },
            }
        });

        if let Some(deployment) = &request.deployment {
            deployment.set_active(unit.clone(), request);
        }
        self.units.write().unwrap().insert(unit.uuid, unit.clone());

        // Print unit information to the console for debugging
        debug!("<red>-----------------------------------</>");
        debug!("<red>New unit added to controller</>");
        debug!("<red>Name: {}</>", unit.name);
        debug!("<red>UUID: {}</>", unit.uuid.to_string());
        debug!("<red>Token: {}</>", unit.auth.token);
        debug!("<red>-----------------------------------</>");

        // Send start request to cloudlet
        // We do this async because the driver chould be running blocking code like network requests
        if let Some(controller) = self.controller.upgrade() {
            let copy = controller.clone();
            controller
                .get_runtime()
                .as_ref()
                .unwrap()
                .spawn_blocking(move || start_thread(copy, unit));
        }

        fn start_thread(controller: ControllerHandle, unit: UnitHandle) {
            if let Some(cloudlet) = unit.cloudlet.upgrade() {
                if let Err(error) = cloudlet.get_inner().start_unit(&unit) {
                    error!(
                        "<red>Failed</> to start unit <red>{}</>: <red>{}</>",
                        unit.name,
                        error
                    );
                    controller.get_units().stop_unit_now(unit);
                }
            }
        }
    }
}

pub struct Unit {
    pub name: String,
    pub uuid: Uuid,
    pub deployment: Option<DeploymentRef>,
    pub cloudlet: WeakCloudletHandle,
    pub allocation: AllocationHandle,

    /* Users */
    pub connected_users: AtomicU32,

    /* Auth */
    pub auth: AuthUnitHandle,

    /* Health and State of the unit */
    pub health: RwLock<Health>,
    pub state: RwLock<State>,
    pub flags: Flags,
    pub rediness: AtomicBool,
}

impl Unit {
    pub fn get_user_count(&self) -> u32 {
        self.connected_users.load(Ordering::Relaxed)
    }
}

pub struct Health {
    pub next_checkin: Instant,
    pub timeout: Duration,
}

impl Health {
    pub fn new(startup_time: Duration, timeout: Duration) -> Self {
        Self {
            next_checkin: Instant::now() + startup_time,
            timeout,
        }
    }
    pub fn reset(&mut self) {
        self.next_checkin = Instant::now() + self.timeout;
    }
    pub fn is_dead(&self) -> bool {
        Instant::now() > self.next_checkin
    }
}

pub struct StartRequest {
    pub canceled: AtomicBool,
    pub when: Option<Instant>,
    pub name: String,
    pub deployment: Option<DeploymentRef>,
    pub cloudlets: Vec<WeakCloudletHandle>,
    pub resources: Resources,
    pub spec: Spec,
    pub priority: i32,
}

pub struct StopRequest {
    pub when: Option<Instant>,
    pub unit: UnitHandle,
}

#[derive(PartialEq, Clone)]
pub enum State {
    Starting,
    Preparing,
    Restarting,
    Running,
    Stopping,
}

pub struct Flags {
    /* Required for the deployment system */
    pub stop: RwLock<Option<Instant>>,
}

#[derive(Clone)]
pub struct DeploymentRef {
    pub unit_id: usize,
    pub deployment: WeakDeploymentHandle,
}

impl DeploymentRef {
    pub fn remove_unit(&self, unit: &UnitHandle) {
        if let Some(deployment) = self.deployment.upgrade() {
            deployment.remove_unit(unit);
        }
    }

    pub fn set_active(&self, unit: UnitHandle, request: &StartRequestHandle) {
        if let Some(deployment) = self.deployment.upgrade() {
            deployment.set_unit_active(unit, request);
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Resources {
    pub memory: u32,
    pub swap: u32,
    pub cpu: u32,
    pub io: u32,
    pub disk: u32,
    pub addresses: u32,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub enum Retention {
    #[serde(rename = "temporary")]
    #[default]
    Temporary,
    #[serde(rename = "permanent")]
    Permanent,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct FallbackPolicy {
    pub enabled: bool,
    pub priority: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Spec {
    pub settings: Vec<KeyValue>,
    pub environment: Vec<KeyValue>,
    pub disk_retention: Retention,
    pub image: String,

    pub fallback: FallbackPolicy,
}
