#![allow(non_camel_case_types)]

use crate::platform::types::{c_int, c_uchar, c_ulong};

pub use crate::header::{bits_pthread_t::pthread_t, bits_pthreadattr_t::pthread_attr_t};

// XXX: https://github.com/eqrion/cbindgen/issues/685
//
// We need to write the opaque types ourselves, and apparently cbindgen doesn't even support
// expanding macros! Instead, we rely on checking that the lengths are correct, when these headers
// are parsed in the regular compilation phase.

/// The `pthread_rwlockattr_t` type provided in [`sys/types.h`](crate::header::sys_types).
#[repr(C)]
pub union pthread_rwlockattr_t {
    __relibc_internal_size: [c_uchar; 1],
    __relibc_internal_align: c_uchar,
}
/// The `pthread_rwlock_t` type provided in [`sys/types.h`](crate::header::sys_types).
#[repr(C)]
pub union pthread_rwlock_t {
    __relibc_internal_size: [c_uchar; 4],
    __relibc_internal_align: c_int,
}
/// The `pthread_barrier_t` type provided in [`sys/types.h`](crate::header::sys_types).
#[repr(C)]
pub union pthread_barrier_t {
    __relibc_internal_size: [c_uchar; 24],
    __relibc_internal_align: c_int,
}
/// The `pthread_barrierattr_t` type provided in [`sys/types.h`](crate::header::sys_types).
#[repr(C)]
pub union pthread_barrierattr_t {
    __relibc_internal_size: [c_uchar; 4],
    __relibc_internal_align: c_int,
}
/// The `pthread_mutex_t` type provided in [`sys/types.h`](crate::header::sys_types).
#[repr(C)]
pub union pthread_mutex_t {
    __relibc_internal_size: [c_uchar; 12],
    __relibc_internal_align: c_int,
}
/// The `pthread_mutexattr_t` type provided in [`sys/types.h`](crate::header::sys_types).
#[repr(C)]
pub union pthread_mutexattr_t {
    __relibc_internal_size: [c_uchar; 20],
    __relibc_internal_align: c_int,
}
/// The `pthread_cond_t` type provided in [`sys/types.h`](crate::header::sys_types).
#[repr(C)]
pub union pthread_cond_t {
    __relibc_internal_size: [c_uchar; 8],
    __relibc_internal_align: c_int,
}
/// The `pthread_condattr_t` type provided in [`sys/types.h`](crate::header::sys_types).
#[repr(C)]
pub union pthread_condattr_t {
    __relibc_internal_size: [c_uchar; 8],
    __relibc_internal_align: c_int,
}
/// The `pthread_spinlock_t` type provided in [`sys/types.h`](crate::header::sys_types).
#[repr(C)]
pub union pthread_spinlock_t {
    __relibc_internal_size: [c_uchar; 4],
    __relibc_internal_align: c_int,
}
/// The `pthread_once_t` type provided in [`sys/types.h`](crate::header::sys_types).
#[repr(C)]
pub union pthread_once_t {
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
            #[allow(clippy::useless_transmute)]
            let b: [u8; core::mem::align_of::<Wrapped>()] = core::mem::transmute(a);
        };
        // TODO: Turn into a macro?
        #[cfg(all(target_os = "redox", feature = "check_against_libc_crate"))]
        const _: () = unsafe {
            use ::__libc_only_for_layout_checks as libc;

            let export = $export { __relibc_internal_align: 0 };
            let _: libc::$export = core::mem::transmute(export.__relibc_internal_size);

            let a = [0_u8; core::mem::align_of::<$export>()];
            let b: [u8; core::mem::align_of::<libc::$export>()] = core::mem::transmute(a);

        };
    }
);
pthread_assert_equal_size!(pthread_rwlock_t, RlctRwlock);
pthread_assert_equal_size!(pthread_rwlockattr_t, RlctRwlockAttr);
pthread_assert_equal_size!(pthread_barrier_t, RlctBarrier);
pthread_assert_equal_size!(pthread_barrierattr_t, RlctBarrierAttr);
pthread_assert_equal_size!(pthread_mutex_t, RlctMutex);
pthread_assert_equal_size!(pthread_mutexattr_t, RlctMutexAttr);
pthread_assert_equal_size!(pthread_cond_t, RlctCond);
pthread_assert_equal_size!(pthread_condattr_t, RlctCondAttr);
pthread_assert_equal_size!(pthread_spinlock_t, RlctSpinlock);
pthread_assert_equal_size!(pthread_once_t, RlctOnce);

/// The `pthread_key_t` type provided in [`sys/types.h`](crate::header::sys_types).
pub type pthread_key_t = c_ulong;
