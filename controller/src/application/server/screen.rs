use anyhow::Result;
use tokio::task::JoinHandle;
use tonic::{async_trait, Status};

pub mod manager;

pub type BoxedScreen = Box<dyn GenericScreen + Send + Sync>;
pub type ScreenPullJoinHandle = JoinHandle<Result<Vec<String>, ScreenError>>;
pub type ScreenWriteJoinHandle = JoinHandle<Result<(), ScreenError>>;

#[async_trait]
pub trait GenericScreen {
    fn is_supported(&self) -> bool;
    fn pull(&self) -> ScreenPullJoinHandle;
    fn write(&self, data: &[u8]) -> ScreenWriteJoinHandle;

    /* Memory */
    async fn cleanup(&mut self) -> Result<()>;
}

pub enum ScreenError {
    Unsupported,
    Error(anyhow::Error),
}

impl From<ScreenError> for Status {
    fn from(val: ScreenError) -> Self {
        match val {
            ScreenError::Unsupported => {
                Status::unimplemented("This feature is not supported by the plugin")
            }
            ScreenError::Error(error) => Status::internal(format!("Error: {error}")),
        }
    }
}
