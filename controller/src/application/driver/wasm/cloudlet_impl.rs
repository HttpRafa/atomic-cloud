use std::sync::{Arc, Weak};

use anyhow::{anyhow, Result};
use tonic::async_trait;
use wasmtime::component::ResourceAny;

use crate::application::{
    auth::AuthUnit,
    cloudlet::{Allocation, HostAndPort},
    driver::GenericCloudlet,
    unit::{
        KeyValue, Resources, Retention, Spec, StartRequest, StartRequestHandle, Unit, UnitHandle,
    },
};

use super::{
    exports::cloudlet::driver::bridge::{self, Address},
    WasmDriver,
};

pub struct WasmCloudlet {
    pub handle: Weak<WasmDriver>,
    pub resource: ResourceAny, // This is delete if the handle is dropped
}

#[async_trait]
impl GenericCloudlet for WasmCloudlet {
    fn allocate_addresses(&self, request: &StartRequestHandle) -> Result<Vec<HostAndPort>> {
        if let Some(driver) = self.handle.upgrade() {
            let mut handle = driver.handle.lock().unwrap();
            let (_, store) = WasmDriver::get_resource_and_store(&mut handle);
            match driver
                .bindings
                .cloudlet_driver_bridge()
                .generic_cloudlet()
                .call_allocate_addresses(store, self.resource, &(request.into()))
            {
                Ok(Ok(addresses)) => addresses
                    .into_iter()
                    .map(|address| Ok(HostAndPort::new(address.host, address.port)))
                    .collect::<Result<Vec<HostAndPort>>>(),
                Ok(Err(error)) => Err(anyhow!(error)),
                Err(error) => Err(error),
            }
        } else {
            Err(anyhow!("Failed to get handle to wasm driver"))
        }
    }

    fn deallocate_addresses(&self, addresses: Vec<HostAndPort>) -> Result<()> {
        if let Some(driver) = self.handle.upgrade() {
            let mut handle = driver.handle.lock().unwrap();
            let (_, store) = WasmDriver::get_resource_and_store(&mut handle);
            driver
                .bindings
                .cloudlet_driver_bridge()
                .generic_cloudlet()
                .call_deallocate_addresses(
                    store,
                    self.resource,
                    &addresses
                        .iter()
                        .map(|address| address.into())
                        .collect::<Vec<Address>>(),
                )
        } else {
            Err(anyhow!("Failed to get handle to wasm driver"))
        }
    }

    fn start_unit(&self, unit: &UnitHandle) -> Result<()> {
        if let Some(driver) = self.handle.upgrade() {
            let mut handle = driver.handle.lock().unwrap();
            let (_, store) = WasmDriver::get_resource_and_store(&mut handle);
            driver
                .bindings
                .cloudlet_driver_bridge()
                .generic_cloudlet()
                .call_start_unit(store, self.resource, &unit.into())
        } else {
            Err(anyhow!("Failed to get handle to wasm driver"))
        }
    }

    fn restart_unit(&self, unit: &UnitHandle) -> Result<()> {
        if let Some(driver) = self.handle.upgrade() {
            let mut handle = driver.handle.lock().unwrap();
            let (_, store) = WasmDriver::get_resource_and_store(&mut handle);
            driver
                .bindings
                .cloudlet_driver_bridge()
                .generic_cloudlet()
                .call_restart_unit(store, self.resource, &unit.into())
        } else {
            Err(anyhow!("Failed to get handle to wasm driver"))
        }
    }

    fn stop_unit(&self, unit: &UnitHandle) -> Result<()> {
        if let Some(driver) = self.handle.upgrade() {
            let mut handle = driver.handle.lock().unwrap();
            let (_, store) = WasmDriver::get_resource_and_store(&mut handle);
            driver
                .bindings
                .cloudlet_driver_bridge()
                .generic_cloudlet()
                .call_stop_unit(store, self.resource, &unit.into())
        } else {
            Err(anyhow!("Failed to get handle to wasm driver"))
        }
    }
}

impl From<&KeyValue> for bridge::KeyValue {
    fn from(val: &KeyValue) -> Self {
        bridge::KeyValue {
            key: val.key.clone(),
            value: val.value.clone(),
        }
    }
}

impl From<&Retention> for bridge::Retention {
    fn from(val: &Retention) -> Self {
        match val {
            Retention::Permanent => bridge::Retention::Permanent,
            Retention::Temporary => bridge::Retention::Temporary,
        }
    }
}

impl From<&Spec> for bridge::Spec {
    fn from(val: &Spec) -> Self {
        bridge::Spec {
            settings: val.settings.iter().map(|setting| setting.into()).collect(),
            environment: val.environment.iter().map(|env| env.into()).collect(),
            disk_retention: (&val.disk_retention).into(),
            image: val.image.clone(),
        }
    }
}

impl From<Resources> for bridge::Resources {
    fn from(val: Resources) -> Self {
        bridge::Resources {
            memory: val.memory,
            swap: val.swap,
            cpu: val.cpu,
            io: val.io,
            disk: val.disk,
            addresses: val.addresses,
        }
    }
}

impl From<Arc<Allocation>> for bridge::Allocation {
    fn from(val: Arc<Allocation>) -> Self {
        bridge::Allocation {
            addresses: val.addresses.iter().map(|address| address.into()).collect(),
            resources: val.resources.clone().into(),
            spec: (&val.spec).into(),
        }
    }
}

impl From<Arc<AuthUnit>> for bridge::Auth {
    fn from(val: Arc<AuthUnit>) -> Self {
        bridge::Auth {
            token: val.token.clone(),
        }
    }
}

impl From<&Arc<Unit>> for bridge::Unit {
    fn from(val: &Arc<Unit>) -> Self {
        bridge::Unit {
            name: val.name.clone(),
            uuid: val.uuid.to_string(),
            deployment: val.deployment.as_ref().map(|deployment| {
                deployment
                    .deployment
                    .upgrade()
                    .expect("Deployment dropped while units of the deployment are still active")
                    .name
                    .clone()
            }),
            allocation: val.allocation.clone().into(),
            auth: val.auth.clone().into(),
        }
    }
}

impl From<&Arc<StartRequest>> for bridge::UnitProposal {
    fn from(val: &Arc<StartRequest>) -> Self {
        bridge::UnitProposal {
            name: val.name.clone(),
            deployment: val.deployment.as_ref().map(|deployment| {
                deployment
                    .deployment
                    .upgrade()
                    .expect("Deployment dropped while units of the deployment are still active")
                    .name
                    .clone()
            }),
            resources: val.resources.clone().into(),
            spec: (&val.spec).into(),
        }
    }
}
