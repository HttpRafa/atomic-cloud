use bitflags::bitflags;

bitflags! {
    pub struct Permissions: u32 {
        const READ = 0b0000_0001;
        const WRITE = 0b0000_0010;
        const EXECUTE = 0b0000_0100;
        const DELETE = 0b0000_1000;
        const CREATE = 0b0001_0000;
        const MODIFY = 0b0010_0000;
        const LIST = 0b0100_0000;
        const ALL = Self::READ.bits() | Self::WRITE.bits() | Self::EXECUTE.bits() | Self::DELETE.bits() | Self::CREATE.bits() | Self::MODIFY.bits() | Self::LIST.bits();
    }
}
