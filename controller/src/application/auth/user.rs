use getset::Getters;

use super::{
    AuthType, GenericAuthorization, OwnedAuthorization, permissions::Permissions,
    server::AuthServer,
};

#[derive(Getters)]
pub struct AdminUser {
    #[getset(get = "pub")]
    username: String,
    #[getset(get = "pub")]
    permissions: Permissions,
}

impl GenericAuthorization for AdminUser {
    fn is_allowed(&self, flag: Permissions) -> bool {
        self.permissions.contains(flag)
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
        AdminUser::create(self.username.clone(), self.permissions.clone())
    }
}

impl AdminUser {
    pub fn create(username: String, permissions: Permissions) -> OwnedAuthorization {
        Box::new(Self {
            username,
            permissions,
        })
    }
}
