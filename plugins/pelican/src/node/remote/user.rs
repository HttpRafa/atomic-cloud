use data::BUser;

use crate::generated::plugin::system::http::Method;

use super::{Endpoint, Remote};

pub mod data;

impl Remote {
    pub fn get_user_by_name(&self, username: &str) -> Option<BUser> {
        self.api_find_on_pages::<BUser>(Method::Get, &Endpoint::Application, "users", |object| {
            object
                .data
                .iter()
                .find(|node| node.attributes.username == username)
                .map(|node| node.attributes.clone())
        })
    }
}
