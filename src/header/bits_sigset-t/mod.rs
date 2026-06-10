use crate::platform::types::c_ulonglong;

/// Integer type of an object used to represent sets of signals.
#[allow(non_camel_case_types)]
pub type sigset_t = c_ulonglong;
