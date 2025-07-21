use futures::executor::block_on;
use std::sync::Arc;
use tonic::{Request, Status, service::Interceptor};

use crate::application::Shared;

#[derive(Clone)]
pub struct AuthInterceptor(pub Arc<Shared>);

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let metadata = request.metadata();
        let token = metadata.get("authorization").and_then(|t| t.to_str().ok());
        if let Some(token) = token {
            match block_on(self.0.auth.has_access(token)) {
                Some(auth) => {
                    request.extensions_mut().insert(auth);
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
