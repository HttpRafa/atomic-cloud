use std::{net::{IpAddr, SocketAddr}, str::FromStr, sync::{Arc, Weak}};

use anyhow::{anyhow, Result};
use tonic::async_trait;
use wasmtime::component::ResourceAny;

use crate::controller::{driver::GenericNode, node::Allocation, server::{Deployment, DriverSetting, Resources, Retention, Server, ServerHandle}};

use super::{exports::node::driver::bridge::{self, Address}, WasmDriver};

pub struct WasmNode {
    pub handle: Weak<WasmDriver>,
    pub resource: ResourceAny, // This is delete if the handle is dropped
}

#[async_trait]
impl GenericNode for WasmNode {
    fn allocate_addresses(&self, amount: u32) -> Result<Vec<SocketAddr>> {
        if let Some(driver) = self.handle.upgrade() {
            let mut handle = driver.handle.lock().unwrap();
            let (_, store) = WasmDriver::get_resource_and_store(&mut handle);
            match driver.bindings.node_driver_bridge().generic_node().call_allocate_addresses(store, self.resource, amount) {
                Ok(Ok(addresses)) => {
                    addresses
                        .into_iter()
                        .map(|address| {
                            let ip = IpAddr::from_str(&address.ip)?;
                            Ok(SocketAddr::new(ip, address.port))
                        })
                        .collect::<Result<Vec<SocketAddr>>>()
                },
                Ok(Err(error)) => Err(anyhow!(error)),
                Err(error) => Err(error),
            }
        } else {
            Err(anyhow!("Failed to get handle to wasm driver"))
        }
    }

    fn deallocate_addresses(&self, addresses: Vec<SocketAddr>) -> Result<()> {
        if let Some(driver) = self.handle.upgrade() {
            let mut handle = driver.handle.lock().unwrap();
            let (_, store) = WasmDriver::get_resource_and_store(&mut handle);
            driver.bindings.node_driver_bridge().generic_node().call_deallocate_addresses(store, self.resource, &addresses.iter().map(|address| address.into()).collect::<Vec<Address>>())
        } else {
            Err(anyhow!("Failed to get handle to wasm driver"))
        }
    }

    fn start_server(&self, server: &ServerHandle) -> Result<()> {
        if let Some(driver) = self.handle.upgrade() {
            let mut handle = driver.handle.lock().unwrap();
            let (_, store) = WasmDriver::get_resource_and_store(&mut handle);
            driver.bindings.node_driver_bridge().generic_node().call_start_server(store, self.resource, &server.into())
        } else {
            Err(anyhow!("Failed to get handle to wasm driver"))
        }
    }

    fn stop_server(&self, server: &ServerHandle) -> Result<()> {
        if let Some(driver) = self.handle.upgrade() {
            let mut handle = driver.handle.lock().unwrap();
            let (_, store) = WasmDriver::get_resource_and_store(&mut handle);
            driver.bindings.node_driver_bridge().generic_node().call_stop_server(store, self.resource, &server.into())
        } else {
            Err(anyhow!("Failed to get handle to wasm driver"))
        }
    }
}

impl From<&DriverSetting> for bridge::DriverSetting {
    fn from(val: &DriverSetting) -> Self {
        bridge::DriverSetting {
            key: val.key.clone(),
            value: val.value.clone(),
        }
    }
}

impl From<&Retention> for bridge::Retention {
    fn from(val: &Retention) -> Self {
        match val {
            Retention::Keep => bridge::Retention::Keep,
            Retention::Delete => bridge::Retention::Delete,
        }
    }
}

impl From<&Deployment> for bridge::Deployment {
    fn from(val: &Deployment) -> Self {
        bridge::Deployment { 
            driver_settings: val.driver_settings.iter().map(|setting| setting.into()).collect(), 
            disk_retention: (&val.disk_retention).into(), 
        }
    }
}

impl From<Resources> for bridge::Resources {
    fn from(val: Resources) -> Self {
        bridge::Resources {
            memory: val.memory,
            cpu: val.cpu,
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
            deployment: (&val.deployment).into(),
        }
    }
}

impl From<&Arc<Server>> for bridge::Server {
    fn from(val: &Arc<Server>) -> Self {
        let group = val.group.upgrade().expect("Failed to get group while trying to convert server to driver representation");
        bridge::Server {
            name: val.name.clone(),
            uuid: val.uuid.to_string(),
            group: group.name.clone(),
            allocation: val.allocation.clone().into(),
        }
    }
}