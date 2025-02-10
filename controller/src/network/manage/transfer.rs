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

pub struct TransferUsersTask(pub Authorization, pub Vec<Uuid>, pub TransferTarget);

#[async_trait]
impl GenericTask for TransferUsersTask {
    async fn run(&mut self, controller: &mut Controller) -> Result<BoxedAny> {
        let mut count: u32 = 0;
        for user in &self.1 {
            let Some(user) = controller.users.get_user_mut(user) else {
                continue;
            };
            let mut transfer = match Transfer::resolve(
                &self.0,
                user,
                &self.2,
                &controller.servers,
                &controller.groups,
            ) {
                Ok(transfer) => transfer,
                Err(error) => return Task::new_err(error.into()),
            };
            if let Err(error) = Transfer::transfer_user(&mut transfer, &controller.shared).await {
                return Task::new_err(error);
            }
            count += 1;
        }
        Task::new_ok(count)
    }
}
