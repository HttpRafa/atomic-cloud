use std::error::Error;
use wasmtime::component::bindgen;
use crate::driver::{GenericDriver, Information};
use crate::node::Node;

bindgen!();

pub struct WasmDriver {
    pub name: String
}

impl DriverImports for WasmDriver {
    fn print(&mut self, message: String) -> () {
        todo!()
    }
}

impl GenericDriver for WasmDriver {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn init(&self) -> Result<Information, Box<dyn Error>> {
        todo!()
    }

    fn init_node(&self, node: &Node) -> Result<bool, Box<dyn Error>> {
        todo!()
    }

    fn stop_server(&self, server: &str) -> Result<(), Box<dyn Error>> {
        todo!()
    }

    fn start_server(&self, server: &str) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}