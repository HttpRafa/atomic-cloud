use std::collections::HashMap;

use data::{BAllocation, BCAllocation};

use crate::generated::plugin::system::http::Method;

use super::{Backend, Endpoint};

pub mod data;

impl Backend {
    pub fn get_allocations_by_server(&self, identifier: &str) -> (BCAllocation, Vec<BCAllocation>) {
        let mut default_allocation = None;
        let mut allocations = Vec::new();
        self.for_each_on_pages::<BCAllocation>(
            Method::Get,
            &Endpoint::Client,
            format!("servers/{}/network/allocations", &identifier).as_str(),
            |response| {
                for allocation in &response.data {
                    if allocation.attributes.is_default {
                        default_allocation = Some(allocation.attributes.clone());
                        continue;
                    }
                    allocations.push(allocation.attributes.clone());
                }
                false
            },
        );
        (
            default_allocation.expect("Expected that the server has min one is_default allocation"),
            allocations,
        )
    }

    pub fn get_free_allocations(
        &self,
        used_allocations: &HashMap<u16, BAllocation>,
        amount: usize,
    ) -> Vec<BAllocation> {
        let mut allocations = Vec::with_capacity(amount);
        self.for_each_on_pages::<BAllocation>(
            Method::Get,
            &Endpoint::Application,
            format!("nodes/{}/allocations", self.node_id).as_str(),
            |response| {
                for allocation in &response.data {
                    if allocation.attributes.assigned
                        || used_allocations.iter().any(|(_, used)| {
                            used.get_host() == allocation.attributes.get_host()
                                && used.port == allocation.attributes.port
                        })
                    {
                        continue;
                    }
                    allocations.push(allocation.attributes.clone());
                    if allocations.len() >= amount {
                        return true;
                    }
                }
                false
            },
        );
        allocations
    }
}
