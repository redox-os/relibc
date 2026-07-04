//! `pthread_attr_t` from `sys/types.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_types.h.html>.

use crate::{
    platform::types::{c_uchar, size_t},
    pthread_assert_equal_size,
};

/// Used to identify a thread attribute object.
#[repr(C)]
pub union pthread_attr_t {
    __relibc_internal_size: [c_uchar; 32],
    __relibc_internal_align: size_t,
}

pthread_assert_equal_size!(pthread_attr_t, RlctAttr);
