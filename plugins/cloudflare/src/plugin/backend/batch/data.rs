use serde::Serialize;

use super::{create::BCreate, delete::BDelete, update::BUpdate};

#[derive(Serialize, Clone)]
pub struct BBatch {
    deletes: Vec<BDelete>,
    patches: Vec<BUpdate>,
    posts: Vec<BCreate>,
}
