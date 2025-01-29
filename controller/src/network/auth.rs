use std::sync::Arc;

use tonic::{service::Interceptor, Request, Status};

use crate::application::{auth::AuthValidator, Controller};

#[derive(Clone)]
pub struct AdminInterceptor {
    pub validator: AuthValidator,
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
pub struct UnitInterceptor {
    pub validator: AuthValidator,
}

impl Interceptor for UnitInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let metadata = request.metadata();
        let token = metadata.get("authorization").and_then(|t| t.to_str().ok());
        match token {
            Some(token) => {
                let unit = self.controller.get_auth().get_unit(token);
                if let Some(unit) = unit {
                    request.extensions_mut().insert(unit);
                    Ok(request)
                } else {
                    Err(Status::unauthenticated("Invalid unit token"))
                }
            }
            None => Err(Status::unauthenticated("No unit token provided")),
        }
    }
}
