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

impl Into<bridge::DriverSetting> for &DriverSetting {
    fn into(self) -> bridge::DriverSetting {
        bridge::DriverSetting {
            key: self.key.clone(),
            value: self.value.clone(),
        }
    }
}

impl Into<bridge::Retention> for &Retention {
    fn into(self) -> bridge::Retention {
        match self {
            Retention::Keep => bridge::Retention::Keep,
            Retention::Delete => bridge::Retention::Delete,
        }
    }
}

impl Into<bridge::Deployment> for &Deployment {
    fn into(self) -> bridge::Deployment {
        bridge::Deployment { 
            driver_settings: self.driver_settings.iter().map(|setting| setting.into()).collect(), 
            disk_retention: (&self.disk_retention).into(), 
        }
    }
}

impl Into<bridge::Resources> for Resources {
    fn into(self) -> bridge::Resources {
        bridge::Resources {
            memory: self.memory,
            cpu: self.cpu,
            disk: self.disk,
            addresses: self.addresses,
        }
    }
}

impl Into<bridge::Allocation> for Arc<Allocation> {
    fn into(self) -> bridge::Allocation {
        bridge::Allocation {
            addresses: self.addresses.iter().map(|address| address.into()).collect(),
            resources: self.resources.clone().into(),
            deployment: (&self.deployment).into(),
        }
    }
}

impl Into<bridge::Server> for &Arc<Server> {
    fn into(self) -> bridge::Server {
        let group = self.group.upgrade().expect("Failed to get group while trying to convert server to driver representation");
        bridge::Server {
            name: self.name.clone(),
            uuid: self.uuid.to_string(),
            group: group.name.clone(),
            allocation: self.allocation.clone().into(),
        }
    }
}