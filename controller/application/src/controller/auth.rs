use super::server::ServerHandle;

pub struct AuthAdmin {
    pub name: String,
    pub token: String,
}

pub struct AuthServer {
    pub server: ServerHandle,
    pub token: String,
}

pub struct Auth {

}

impl Auth {
    pub fn load_all() -> Self {
        Self {}
    }

    pub fn get_admin(&self, token: &str) -> Option<AuthAdmin> {
        None
    }

    pub fn get_server(&self, token: &str) -> Option<AuthServer> {
        None
    }
}