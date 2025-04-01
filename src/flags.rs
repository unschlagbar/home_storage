
#[repr(u8)]
#[derive(Clone, Copy)]
#[allow(unused)]
pub enum PacketFlag {
    Save = 0,
    Remove = 1,
    Get = 2,
}

impl From<u8> for PacketFlag {

    fn from(value: u8) -> Self {
        unsafe { *(&value as *const u8 as *const PacketFlag) }
    }
}