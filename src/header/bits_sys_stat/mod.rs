use crate::platform::types::{c_long, c_longlong};

/// Used for file block counts.
#[allow(non_camel_case_types)]
pub type blkcnt_t = c_longlong;

/// Used for block sizes.
#[allow(non_camel_case_types)]
pub type blksize_t = c_long;
