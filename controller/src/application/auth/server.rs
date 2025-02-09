use getset::Getters;
use uuid::Uuid;

use super::{user::AdminUser, AuthType, GenericAuthorization, OwnedAuthorization};

#[derive(Getters)]
pub struct AuthServer {
    #[getset(get = "pub")]
    uuid: Uuid,
}

impl GenericAuthorization for AuthServer {
    fn is_allowed(&self, _flag: u32) -> bool {
        true
    }

    fn get_user(&self) -> Option<&AdminUser> {
        None
    }
    fn get_server(&self) -> Option<&AuthServer> {
        Some(self)
    }
    fn is_type(&self, auth: AuthType) -> bool {
        auth == AuthType::Server
    }

    fn recreate(&self) -> OwnedAuthorization {
        AuthServer::create(self.uuid)
    }
}

impl AuthServer {
    pub fn create(uuid: Uuid) -> OwnedAuthorization {
        Box::new(Self { uuid })
    }
}
