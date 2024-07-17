use std::fmt::{Display, Formatter};

pub enum Stage {
    Stable,
    Beta,
    Alpha,
}

impl Display for Stage {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Stage::Stable => write!(formatter, "stable"),
            Stage::Beta => write!(formatter, "beta"),
            Stage::Alpha => write!(formatter, "alpha"),
        }
    }
}

pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub stage: Stage,
}

impl Display for Version {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}.{}.{}-{}",
            self.major, self.minor, self.patch, self.stage
        )
    }
}
