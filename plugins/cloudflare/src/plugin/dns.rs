pub struct NewRecord {
    pub name: String,
    pub priority: u16,
    pub weight: u16,
    pub port: u16,
    pub target: String,
}

pub struct Record {
    pub server: String,
    pub record: String,
}
