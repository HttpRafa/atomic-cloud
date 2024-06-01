use serde::Deserialize;

#[derive(Deserialize)]
pub struct BNodes {
    pub data: Vec<BNodeObject>,
}

#[derive(Deserialize)]
pub struct BNodeObject {
    pub attributes: BNode
}

#[derive(Deserialize)]
pub struct BNode {
    pub name: String,
}