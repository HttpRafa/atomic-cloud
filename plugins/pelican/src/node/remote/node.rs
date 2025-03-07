use data::BNode;

use crate::generated::plugin::system::http::Method;

use super::{Endpoint, Remote};

pub mod data;

impl Remote {
    pub fn get_node_by_name(&self, name: &str) -> Option<BNode> {
        self.api_find_on_pages::<BNode>(Method::Get, &Endpoint::Application, "nodes", |object| {
            object
                .data
                .iter()
                .find(|node| node.attributes.name == name)
                .map(|node| node.attributes.clone())
        })
    }
}
