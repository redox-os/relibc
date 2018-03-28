use platform::types::*;

pub const PROT_READ: c_int = 0x1;
pub const PROT_WRITE: c_int = 0x2;
pub const PROT_EXEC: c_int = 0x4;
pub const PROT_NONE: c_int = 0x0;

pub const MAP_SHARED: c_int = 0x1;
pub const MAP_PRIVATE: c_int = 0x2;
