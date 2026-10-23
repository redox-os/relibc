use crate::platform::types::{c_long, c_longlong};

/// Used for file block counts.
pub type blkcnt_t = c_longlong;

/// Used for block sizes.
pub type blksize_t = c_long;
