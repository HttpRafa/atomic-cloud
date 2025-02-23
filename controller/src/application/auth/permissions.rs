use bitflags::bitflags;

bitflags! {
    pub struct Permissions: u32 {
        const READ = 1;
        const WRITE = 1 << 1;
        const EXECUTE = 1 << 2;
        const DELETE = 1 << 3;
        const CREATE = 1 << 4;
        const MODIFY = 1 << 5;
        const LIST = 1 << 6;
        const ALL = Self::READ.bits() | Self::WRITE.bits() | Self::EXECUTE.bits() | Self::DELETE.bits() | Self::CREATE.bits() | Self::MODIFY.bits() | Self::LIST.bits();
    }
}
