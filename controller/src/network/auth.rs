use futures::executor::block_on;
use std::sync::Arc;
use tonic::{service::Interceptor, Request, Status};

use crate::application::auth::{service::AuthService, Authorization};

#[derive(Clone)]
pub struct AuthInterceptor(pub Arc<AuthService>);

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let metadata = request.metadata();
        let token = metadata.get("authorization").and_then(|t| t.to_str().ok());
        if let Some(token) = token {
            match block_on(self.0.has_access(token)) {
                Some(Authorization::User(user)) => {
                    request.extensions_mut().insert(user);
                    Ok(request)
                }
                Some(Authorization::Server(server)) => {
                    request.extensions_mut().insert(server);
                    Ok(request)
                }
                _ => Err(Status::unauthenticated(
                    "Invalid authorization token provided",
                )),
            }
        } else {
            Err(Status::unauthenticated("No authorization token provided"))
        }
    }
}
