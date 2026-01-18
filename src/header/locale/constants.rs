use crate::platform::types::c_int;

pub const LC_COLLATE: c_int = 0;
pub const LC_CTYPE: c_int = 1;
pub const LC_MESSAGES: c_int = 2;
pub const LC_MONETARY: c_int = 3;
pub const LC_NUMERIC: c_int = 4;
pub const LC_TIME: c_int = 5;
pub const LC_ALL: c_int = 6;
pub const LC_COLLATE_MASK: c_int = 0x1;
pub const LC_CTYPE_MASK: c_int = 0x2;
pub const LC_MESSAGES_MASK: c_int = 0x4;
pub const LC_MONETARY_MASK: c_int = 0x8;
pub const LC_NUMERIC_MASK: c_int = 0x10;
pub const LC_TIME_MASK: c_int = 0x20;
pub const LC_ALL_MASK: c_int = 0b111111;
