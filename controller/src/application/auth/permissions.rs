use bitflags::bitflags;
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
    #[serde(transparent)]
    pub struct Permissions: u32 {
        const REQUEST_STOP = 1;

        const SET_RESOURCE = 1 << 1;
        const DELETE_RESOURCE = 1 << 2;

        const CREATE_NODE = 1 << 3;
        const UPDATE_NODE = 1 << 4;
        const GET_NODE = 1 << 5;

        const CREATE_GROUP = 1 << 6;
        const UPDATE_GROUP = 1 << 7;
        const GET_GROUP = 1 << 8;

        const SCHEDULE_SERVER = 1 << 9;
        const GET_SERVER = 1 << 10;

        const WRITE_TO_SCREEN = 1 << 11;
        const READ_SCREEN = 1 << 12;

        const GET_USER = 1 << 13;

        const TRANSFER_USER = 1 << 14;

        const READ_POWER_EVENTS = 1 << 15;
        const READ_READY_EVENTS = 1 << 16;

        const LIST = 1 << 17;

        const ALL = Self::REQUEST_STOP.bits() | Self::SET_RESOURCE.bits() | Self::DELETE_RESOURCE.bits() | Self::CREATE_NODE.bits() | Self::UPDATE_NODE.bits() | Self::GET_NODE.bits() | Self::CREATE_GROUP.bits() | Self::UPDATE_GROUP.bits() | Self::GET_GROUP.bits() | Self::SCHEDULE_SERVER.bits() | Self::GET_SERVER.bits() | Self::WRITE_TO_SCREEN.bits() | Self::READ_SCREEN.bits() | Self::GET_USER.bits() | Self::TRANSFER_USER.bits() | Self::READ_POWER_EVENTS.bits() | Self::READ_READY_EVENTS.bits() | Self::LIST.bits();
    }
}
