use std::fmt::{Display, Formatter};

pub struct Version {
    pub(crate) major: u16,
    pub(crate) minor: u16,
    pub(crate) patch: u16
}

impl Display for Version {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}