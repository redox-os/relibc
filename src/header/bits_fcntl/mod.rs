use crate::platform::types::c_int;

/// Seek relative to start-of-file.
pub const SEEK_SET: c_int = 0;
/// Seek relative to current position.
pub const SEEK_CUR: c_int = 1;
/// Seek relative to end-of-file.
pub const SEEK_END: c_int = 2;
