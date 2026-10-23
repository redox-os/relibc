use crate::platform::types::c_uint;

/// Used as a general identifier; can be used to contain at least a pid_t, uid_t, or gid_t.
pub type id_t = c_uint;
