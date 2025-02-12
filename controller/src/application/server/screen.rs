use anyhow::Result;
use tokio::task::JoinHandle;
use tonic::Status;

pub mod manager;

pub type ScreenJoinHandle = JoinHandle<Result<Vec<String>, PullError>>;

pub trait GenericScreen {
    fn is_supported(&self) -> bool;
    fn pull(&self) -> ScreenJoinHandle;
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
