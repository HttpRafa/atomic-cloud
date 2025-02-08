use std::sync::Arc;

use permissions::Permission;
use server::AuthServer;
use user::AdminUser;

pub mod permissions;
pub mod manager;

pub mod server;
pub mod user;

const DEFAULT_ADMIN_USERNAME: &str = "admin";

pub type AuthToken = String;
pub type OwnedAuthorization = Box<dyn GenericAuthorization + Send + Sync>;
pub type Authorization = Arc<OwnedAuthorization>;

pub trait GenericAuthorization {
    fn get_server(&self) -> Option<&AuthServer>;
    fn get_user(&self) -> Option<&AdminUser>;
    fn is_type(&self, auth: AuthType) -> bool;

    fn is_allowed(&self, permission: Permission) -> bool;

    fn recreate(&self) -> OwnedAuthorization;
}

#[derive(PartialEq)]
pub enum AuthType {
    User,
    Server,
}

#[derive(PartialEq)]
pub enum ActionResult {
    Allowed,
    Denied,
}
