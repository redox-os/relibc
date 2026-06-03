use crate::platform::types::c_void;

/// Null pointer constant.
#[expect(clippy::zero_ptr)]
pub const NULL: *mut c_void = 0 as *mut c_void;
