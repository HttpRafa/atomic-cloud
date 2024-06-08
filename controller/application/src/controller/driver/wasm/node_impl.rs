use std::{net::{IpAddr, SocketAddr}, str::FromStr, sync::Weak};

use anyhow::{anyhow, Result};
use tonic::async_trait;
use wasmtime::component::ResourceAny;

use crate::controller::driver::GenericNode;

use super::WasmDriver;

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
            return match driver.bindings.node_driver_bridge().generic_node().call_allocate_addresses(store, self.resource, amount).await {
                Ok(addresses) => match addresses {
                    Ok(addresses) => {
                        let mut result = Vec::with_capacity(addresses.len());
                        for address in addresses {
                            let ip = IpAddr::from_str(&address.ip)?;
                            result.push(SocketAddr::new(ip, address.port));
                        }
                        Ok(result)
                    },
                    Err(error) => Err(anyhow!(error)),
                },
                Err(error) => Err(error),
            }
        }
        Err(anyhow!("Failed to get handle to wasm driver"))
    }
}