use anyhow::Result;
use tokio::task::JoinHandle;
use tonic::{async_trait, Status};

pub mod manager;

pub type BoxedScreen = Box<dyn GenericScreen + Send + Sync>;
pub type ScreenJoinHandle = JoinHandle<Result<Vec<String>, PullError>>;

#[async_trait]
pub trait GenericScreen {
    fn is_supported(&self) -> bool;
    fn pull(&self) -> ScreenJoinHandle;

    /* Memory */
    async fn drop_resources(&mut self) -> Result<()>;
}

pub enum PullError {
    Unsupported,
    Error(anyhow::Error),
}

impl From<PullError> for Status {
    fn from(val: PullError) -> Self {
        match val {
            PullError::Unsupported => {
                Status::unimplemented("This feature is not supported by the plugin")
            }
            PullError::Error(error) => Status::internal(format!("Error: {error}")),
        }
    }
}
