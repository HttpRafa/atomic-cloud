use std::{net::{IpAddr, SocketAddr}, str::FromStr, sync::{Arc, Weak}};

use anyhow::{anyhow, Result};
use tonic::async_trait;
use wasmtime::component::ResourceAny;

use crate::controller::{driver::GenericNode, node::Allocation, server::{DeploySetting, Resources, Retention, Server, ServerHandle}};

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
            let (_, store) = handle.get();
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
            let (_, store) = handle.get();
            driver.bindings.node_driver_bridge().generic_node().call_deallocate_addresses(store, self.resource, &addresses.iter().map(|address| address.into()).collect::<Vec<Address>>())
        } else {
            Err(anyhow!("Failed to get handle to wasm driver"))
        }
    }

    fn start_server(&self, server: &ServerHandle) -> Result<()> {
        if let Some(driver) = self.handle.upgrade() {
            let mut handle = driver.handle.lock().unwrap();
            let (_, store) = handle.get();
            driver.bindings.node_driver_bridge().generic_node().call_start_server(store, self.resource, &server.into())
        } else {
            Err(anyhow!("Failed to get handle to wasm driver"))
        }
    }

    fn stop_server(&self, server: &ServerHandle) -> Result<()> {
        if let Some(driver) = self.handle.upgrade() {
            let mut handle = driver.handle.lock().unwrap();
            let (_, store) = handle.get();
            driver.bindings.node_driver_bridge().generic_node().call_stop_server(store, self.resource, &server.into())
        } else {
            Err(anyhow!("Failed to get handle to wasm driver"))
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

impl Into<bridge::DeploySetting> for &DeploySetting {
    fn into(self) -> bridge::DeploySetting {
        match self {
            DeploySetting::Image(image) => bridge::DeploySetting::Image(image.clone()),
            DeploySetting::DiskRetention(retention) => bridge::DeploySetting::DiskRetention(retention.into()),
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
            deployment: self.deployment.iter().map(|setting: &DeploySetting| setting.into()).collect(),
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