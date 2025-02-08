use std::collections::HashMap;

use tokio::sync::RwLock;

use crate::network::client::{ChannelMsg, TransferMsg};

use super::Subscription;

pub struct SubscriptionManager {
    transfer: RwLock<HashMap<String, Vec<Subscription<TransferMsg>>>>,
    channel: RwLock<HashMap<String, Vec<Subscription<ChannelMsg>>>>,
}

impl SubscriptionManager {
    
}