use std::sync::Arc;

use tonic::{service::Interceptor, Request, Status};

use crate::application::Controller;

#[derive(Clone)]
pub struct AdminInterceptor {
    pub controller: Arc<Controller>,
}

impl Interceptor for AdminInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let metadata = request.metadata();
        let token = metadata.get("authorization").and_then(|t| t.to_str().ok());
        match token {
            Some(token) => {
                let user = self.controller.get_auth().get_user(token);
                if let Some(user) = user {
                    request.extensions_mut().insert(user);
                    Ok(request)
                } else {
                    Err(Status::unauthenticated("Invalid user token"))
                }
            }
            None => Err(Status::unauthenticated("No user token provided")),
        }
    }
}

#[derive(Clone)]
pub struct ServerInterceptor {
    pub controller: Arc<Controller>,
}

impl Interceptor for ServerInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let metadata = request.metadata();
        let token = metadata.get("authorization").and_then(|t| t.to_str().ok());
        match token {
            Some(token) => {
                let server = self.controller.get_auth().get_server(token);
                if let Some(server) = server {
                    request.extensions_mut().insert(server);
                    Ok(request)
                } else {
                    Err(Status::unauthenticated("Invalid server token"))
                }
            }
            None => Err(Status::unauthenticated("No server token provided")),
        }
    }
}
