use getset::Getters;

use super::{
    permissions::Permission, server::AuthServer, AuthType, GenericAuthorization, OwnedAuthorization,
};

#[derive(Getters)]
pub struct AdminUser {
    #[getset(get = "pub")]
    username: String,
}

impl GenericAuthorization for AdminUser {
    fn is_allowed(&self, _permission: Permission) -> bool {
        true // TODO: Implement permission check
    }

    fn get_user(&self) -> Option<&AdminUser> {
        Some(self)
    }
    fn get_server(&self) -> Option<&AuthServer> {
        None
    }
    fn is_type(&self, auth: AuthType) -> bool {
        auth == AuthType::User
    }

    fn recreate(&self) -> OwnedAuthorization {
        AdminUser::create(self.username.clone())
    }
}

impl AdminUser {
    pub fn create(username: String) -> OwnedAuthorization {
        Box::new(Self { username })
    }
}
