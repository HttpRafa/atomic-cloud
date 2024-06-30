use std::sync::Arc;

use tonic::{service::Interceptor, Request, Status};

use crate::controller::Controller;

#[derive(Clone)]
pub struct AdminInterceptor {
    pub controller: Arc<Controller>,
}

impl Interceptor for AdminInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        let metadata = request.metadata();
        let token = metadata.get("authorization").and_then(|t| t.to_str().ok());
        match token {
            Some(token) => {
                let admin = self.controller.get_auth().get_admin(token);
                if let Some(_admin) = admin {
                    // TODO: Add admin to request that the method knows which admin is calling
                    Ok(request)
                } else {
                    Err(Status::unauthenticated("Invalid admin token"))
                }
            }
            None => Err(Status::unauthenticated("No admin token provided")),
        }
    }
}

#[derive(Clone)]
pub struct ServerInterceptor {
    pub controller: Arc<Controller>,
}

impl Interceptor for ServerInterceptor {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        let metadata = request.metadata();
        let token = metadata.get("authorization").and_then(|t| t.to_str().ok());
        match token {
            Some(token) => {
                let server = self.controller.get_auth().get_server(token);
                if let Some(_server) = server {
                    // TODO: Add server to request that the method knows which server is calling
                    Ok(request)
                } else {
                    Err(Status::unauthenticated("Invalid server token"))
                }
            }
            None => Err(Status::unauthenticated("No server token provided")),
        }
    }
}