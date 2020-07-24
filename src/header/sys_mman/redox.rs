use crate::platform::types::*;

pub const PROT_NONE: c_int = 0x0000;
pub const PROT_EXEC: c_int = 0x0001;
pub const PROT_WRITE: c_int = 0x0002;
pub const PROT_READ: c_int = 0x0004;

pub const MAP_FIXED: c_int = 0x0004;
pub const MAP_FIXED_NOREPLACE: c_int = 0x000C;
