//! `pthread_once_t` for `sys/types.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_types.h.html>.

use crate::{
    platform::types::{c_int, c_uchar},
    pthread_assert_equal_size,
};

/// Used for dynamic package initialization.
#[repr(C)]
pub union pthread_once_t {
    __relibc_internal_size: [c_uchar; 4],
    __relibc_internal_align: c_int,
}

pthread_assert_equal_size!(pthread_once_t, RlctOnce);
