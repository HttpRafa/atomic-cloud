use tonic::Status;

pub enum DeleteResourceError {
    StillActive,
    StillInUse,
    NotFound,
    Error(anyhow::Error),
}

pub enum CreateResourceError {
    RequiredNodeNotLoaded,
    RequiredPluginNotLoaded,
    AlreadyExists,
    Error(anyhow::Error),
}

pub enum UpdateResourceError {
    RequiredNodeNotLoaded,
    RequiredPluginNotLoaded,
    NotFound,
    Error(anyhow::Error),
}

impl From<DeleteResourceError> for Status {
    fn from(val: DeleteResourceError) -> Self {
        match val {
            DeleteResourceError::StillActive => {
                Status::unavailable("Resource is still set to active")
            }
            DeleteResourceError::StillInUse => Status::unavailable("Resource is still in use"),
            DeleteResourceError::NotFound => Status::not_found("Resource not found"),
            DeleteResourceError::Error(error) => Status::internal(format!("Error: {error}")),
        }
    }
}

impl From<CreateResourceError> for Status {
    fn from(val: CreateResourceError) -> Self {
        match val {
            CreateResourceError::RequiredNodeNotLoaded => {
                Status::failed_precondition("Required node is not loaded")
            }
            CreateResourceError::RequiredPluginNotLoaded => {
                Status::failed_precondition("Required plugin is not loaded")
            }
            CreateResourceError::AlreadyExists => Status::already_exists("Resource already exists"),
            CreateResourceError::Error(error) => Status::internal(format!("Error: {error}")),
        }
    }
}

impl From<UpdateResourceError> for Status {
    fn from(val: UpdateResourceError) -> Self {
        match val {
            UpdateResourceError::RequiredNodeNotLoaded => {
                Status::failed_precondition("Required node is not loaded")
            }
            UpdateResourceError::RequiredPluginNotLoaded => {
                Status::failed_precondition("Required plugin is not loaded")
            }
            UpdateResourceError::NotFound => Status::not_found("Resource not found"),
            UpdateResourceError::Error(error) => Status::internal(format!("Error: {error}")),
        }
    }
}
