use std::sync::Arc;

use permissions::Permissions;
use server::AuthServer;
use user::AdminUser;

pub mod manager;
pub mod permissions;

pub mod server;
pub mod user;

const DEFAULT_ADMIN_USERNAME: &str = "admin";
const DEFAULT_ADMIN_PERMISSIONS: Permissions = Permissions::ALL;

pub type AuthToken = String;
pub type OwnedAuthorization = Box<dyn GenericAuthorization + Send + Sync>;
pub type Authorization = Arc<OwnedAuthorization>;

pub trait GenericAuthorization {
    fn get_server(&self) -> Option<&AuthServer>;
    #[allow(unused)]
    fn get_user(&self) -> Option<&AdminUser>;
    fn is_type(&self, auth: AuthType) -> bool;

    fn is_allowed(&self, flag: Permissions) -> bool;

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
