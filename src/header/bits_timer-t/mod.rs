use crate::platform::types::c_void;

/// Used for timer ID returned by timer_create()
#[allow(non_camel_case_types)]
pub type timer_t = *mut c_void;
