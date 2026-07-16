//! pthread types for `sys/types.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_types.h.html>.

#![allow(non_camel_case_types)]

use crate::platform::types::{c_int, c_uchar};

pub use crate::header::{
    bits_pthread_t::pthread_t,
    bits_pthreadattr_t::pthread_attr_t,
    bits_pthreadonce_t::pthread_once_t,
    bits_threads::{pthread_cond_t, pthread_key_t, pthread_mutex_t},
};

// XXX: https://github.com/eqrion/cbindgen/issues/685
//
// We need to write the opaque types ourselves, and apparently cbindgen doesn't even support
// expanding macros! Instead, we rely on checking that the lengths are correct, when these headers
// are parsed in the regular compilation phase.

/// Used for read/write lock attributes.
#[repr(C)]
pub union pthread_rwlockattr_t {
    __relibc_internal_size: [c_uchar; 1],
    __relibc_internal_align: c_uchar,
}
/// Used for read-write locks.
#[repr(C)]
pub union pthread_rwlock_t {
    __relibc_internal_size: [c_uchar; 4],
    __relibc_internal_align: c_int,
}
/// Used to identify a barrier.
#[repr(C)]
pub union pthread_barrier_t {
    __relibc_internal_size: [c_uchar; 24],
    __relibc_internal_align: c_int,
}
/// Used to define a barrier attributes object.
#[repr(C)]
pub union pthread_barrierattr_t {
    __relibc_internal_size: [c_uchar; 4],
    __relibc_internal_align: c_int,
}
/// Used to identify a mutex attribute object.
#[repr(C)]
pub union pthread_mutexattr_t {
    __relibc_internal_size: [c_uchar; 20],
    __relibc_internal_align: c_int,
}
/// Used to identify a condition attribute object.
#[repr(C)]
pub union pthread_condattr_t {
    __relibc_internal_size: [c_uchar; 8],
    __relibc_internal_align: c_int,
}
/// Used to identify a spin lock.
#[repr(C)]
pub union pthread_spinlock_t {
    __relibc_internal_size: [c_uchar; 4],
    __relibc_internal_align: c_int,
}

#[macro_export]
macro_rules! pthread_assert_equal_size(
    ($export:ident, $wrapped:ident) => {
        const _: () = unsafe {
            type Wrapped = $crate::header::pthread::$wrapped;

            // Fail at compile-time if sizes differ.

            // TODO: Is this UB?
            let export = $export { __relibc_internal_align: 0 };
            let _: Wrapped = core::mem::transmute(export.__relibc_internal_size);

            // Fail at compile-time if alignments differ.
            let a = [0_u8; core::mem::align_of::<$export>()];
            #[expect(clippy::useless_transmute)]
            let b: [u8; core::mem::align_of::<Wrapped>()] = core::mem::transmute(a);
        };
        // TODO: Turn into a macro?
        #[cfg(all(target_os = "redox", feature = "check_against_libc_crate"))]
        const _: () = unsafe {
            use ::__libc_only_for_layout_checks as libc;

            let export = $export { __relibc_internal_align: 0 };
            let _: libc::$export = core::mem::transmute(export.__relibc_internal_size);

            let a = [0_u8; core::mem::align_of::<$export>()];
            #[expect(clippy::useless_transmute)]
            let b: [u8; core::mem::align_of::<libc::$export>()] = core::mem::transmute(a);

        };
    }
);
pthread_assert_equal_size!(pthread_rwlock_t, RlctRwlock);
pthread_assert_equal_size!(pthread_rwlockattr_t, RlctRwlockAttr);
pthread_assert_equal_size!(pthread_barrier_t, RlctBarrier);
pthread_assert_equal_size!(pthread_barrierattr_t, RlctBarrierAttr);
pthread_assert_equal_size!(pthread_mutexattr_t, RlctMutexAttr);
pthread_assert_equal_size!(pthread_condattr_t, RlctCondAttr);
pthread_assert_equal_size!(pthread_spinlock_t, RlctSpinlock);
