use platform::types::*;

pub const PROT_READ: c_int = 0;
pub const PROT_WRITE: c_int = 0;
pub const PROT_EXEC: c_int = 0;
pub const PROT_NONE: c_int = 0;

pub const MAP_SHARED: c_int = 0;
pub const MAP_PRIVATE: c_int = 0;
pub const MAP_ANON: c_int = 1;
pub const MAP_ANONYMOUS: c_int = MAP_ANON;
