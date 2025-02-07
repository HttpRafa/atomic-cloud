use anyhow::Result;
use tonic::async_trait;
use uuid::Uuid;

use crate::{
    application::{
        auth::Authorization,
        user::transfer::{Transfer, TransferTarget},
        Controller,
    },
    task::{BoxedAny, GenericTask, Task},
};

pub struct TransferUsersTask {
    pub auth: Authorization,
    pub uuids: Vec<Uuid>,
    pub target: TransferTarget,
}

#[async_trait]
impl GenericTask for TransferUsersTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        for user in &self.uuids {
            let user = match controller.users.get_user(user) {
                Some(user) => user,
                None => continue,
            };
            let transfer = match Transfer::resolve(
                &self.auth,
                user,
                &self.target,
                &controller.servers,
                &controller.groups,
            ) {
                Ok(transfer) => transfer,
                Err(error) => return Task::new_err(error.into()),
            };
        }
        Task::new_empty()
    }
}
