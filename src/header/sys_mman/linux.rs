use crate::platform::types::*;

pub const PROT_READ: c_int = 0x0001;
pub const PROT_WRITE: c_int = 0x0002;
pub const PROT_EXEC: c_int = 0x0004;
pub const PROT_NONE: c_int = 0x0000;

pub const MAP_FIXED: c_int = 0x0010;
pub const MAP_FIXED_NOREPLACE: c_int = 0x100000;
pub const MAP_POPULATE: c_int = 0x008000;
pub const MAP_HUGETLB: c_int = 0x40000;
pub const MAP_NORESERVE: c_int = 0x4000;

pub const MADV_HUGEPAGE: c_int = 14;
pub const MADV_NOHUGEPAGE: c_int = 15;
pub const MADV_DONTDUMP: c_int = 16;
pub const MADV_DODUMP: c_int = 17;
