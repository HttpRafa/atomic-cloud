use super::plugin::WrappedNode;

pub mod manager;

pub struct Node {
    inner: WrappedNode,
}