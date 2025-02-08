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
        let mut count: u32 = 0;
        for user in &self.uuids {
            let user = match controller.users.get_user_mut(user) {
                Some(user) => user,
                None => continue,
            };
            let mut transfer = match Transfer::resolve(
                &self.auth,
                user,
                &self.target,
                &controller.servers,
                &controller.groups,
            ) {
                Ok(transfer) => transfer,
                Err(error) => return Task::new_err(error.into()),
            };
            if let Err(error) = Transfer::transfer_user(&mut transfer, &controller.shared).await {
                return Task::new_err(error);
            } else {
                count += 1;
            }
        }
        Task::new_ok(count)
    }
}
