use uuid::Uuid;

pub mod validator;

const DEFAULT_ADMIN_USERNAME: &str = "admin";

pub type AuthToken = String;

#[derive(Clone)]
pub enum Authorization {
    User(AdminUser),
    Server(Uuid),
}

#[derive(Clone)]
pub struct AdminUser {
    pub username: String,
}

impl AdminUser {
    pub fn new(username: String) -> Self {
        Self { username }
    }
}
