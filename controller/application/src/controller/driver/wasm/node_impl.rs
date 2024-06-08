use std::{net::{IpAddr, SocketAddr}, str::FromStr, sync::Weak};

use anyhow::{anyhow, Result};
use tonic::async_trait;
use wasmtime::component::ResourceAny;

use crate::controller::{driver::GenericNode, server::ServerHandle};

use super::{exports::node::driver::bridge::Address, WasmDriver};

pub struct WasmNode {
    pub handle: Weak<WasmDriver>,
    pub resource: ResourceAny, // This is delete if the handle is dropped
}

#[async_trait]
impl GenericNode for WasmNode {
    async fn allocate_addresses(&self, amount: u32) -> Result<Vec<SocketAddr>> {
        if let Some(driver) = self.handle.upgrade() {
            let mut handle = driver.handle.lock().await;
            let (_, store) = handle.get();
            match driver.bindings.node_driver_bridge().generic_node().call_allocate_addresses(store, self.resource, amount).await {
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

    async fn deallocate_addresses(&self, addresses: Vec<SocketAddr>) -> Result<()> {
        if let Some(driver) = self.handle.upgrade() {
            let mut handle = driver.handle.lock().await;
            let (_, store) = handle.get();
            driver.bindings.node_driver_bridge().generic_node().call_deallocate_addresses(store, self.resource, &addresses.iter().map(|address| address.into()).collect::<Vec<Address>>()).await
        } else {
            Err(anyhow!("Failed to get handle to wasm driver"))
        }
    }

    async fn start_server(&self, _server: &ServerHandle) -> Result<()> {
        Err(anyhow!("Not implemented yet"))
    }
}