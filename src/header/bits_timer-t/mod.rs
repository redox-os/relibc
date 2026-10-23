use crate::platform::types::c_void;

/// Used for timer ID returned by timer_create()
pub type timer_t = *mut c_void;
