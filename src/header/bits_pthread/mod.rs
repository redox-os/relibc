#![allow(non_camel_case_types)]

use crate::platform::types::*;

// XXX: https://github.com/eqrion/cbindgen/issues/685
//
// We need to write the opaque types ourselves, and apparently cbindgen doesn't even support
// expanding macros! Instead, we rely on checking that the lengths are correct, when these headers
// are parsed in the regular compilation phase.

#[repr(C)]
pub union pthread_attr_t {
    __relibc_internal_size: [c_uchar; 32],
    __relibc_internal_align: size_t,
}
#[repr(C)]
pub union pthread_rwlockattr_t {
    __relibc_internal_size: [c_uchar; 1],
    __relibc_internal_align: c_uchar,
}
#[repr(C)]
pub union pthread_rwlock_t {
    __relibc_internal_size: [c_uchar; 4],
    __relibc_internal_align: c_int,
}
#[repr(C)]
pub union pthread_barrier_t {
    __relibc_internal_size: [c_uchar; 24],
    __relibc_internal_align: c_int,
}
#[repr(C)]
pub union pthread_barrierattr_t {
    __relibc_internal_size: [c_uchar; 4],
    __relibc_internal_align: c_int,
}
#[repr(C)]
pub union pthread_mutex_t {
    __relibc_internal_size: [c_uchar; 12],
    __relibc_internal_align: c_int,
}
#[repr(C)]
pub union pthread_mutexattr_t {
    __relibc_internal_size: [c_uchar; 20],
    __relibc_internal_align: c_int,
}
#[repr(C)]
pub union pthread_cond_t {
    __relibc_internal_size: [c_uchar; 8],
    __relibc_internal_align: c_int,
}
#[repr(C)]
pub union pthread_condattr_t {
    __relibc_internal_size: [c_uchar; 8],
    __relibc_internal_align: c_int,
}
#[repr(C)]
pub union pthread_spinlock_t {
    __relibc_internal_size: [c_uchar; 4],
    __relibc_internal_align: c_int,
}
#[repr(C)]
pub union pthread_once_t {
    __relibc_internal_size: [c_uchar; 4],
    __relibc_internal_align: c_int,
}

macro_rules! assert_equal_size(
    ($export:ident, $wrapped:ident) => {
        const _: () = unsafe {
            type Wrapped = crate::header::pthread::$wrapped;

            // Fail at compile-time if sizes differ.

            // TODO: Is this UB?
            let export = $export { __relibc_internal_align: 0 };
            let _: Wrapped = core::mem::transmute(export.__relibc_internal_size);

            // Fail at compile-time if alignments differ.
            let a = [0_u8; core::mem::align_of::<$export>()];
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
assert_equal_size!(pthread_attr_t, RlctAttr);
assert_equal_size!(pthread_rwlock_t, RlctRwlock);
assert_equal_size!(pthread_rwlockattr_t, RlctRwlockAttr);
assert_equal_size!(pthread_barrier_t, RlctBarrier);
assert_equal_size!(pthread_barrierattr_t, RlctBarrierAttr);
assert_equal_size!(pthread_mutex_t, RlctMutex);
assert_equal_size!(pthread_mutexattr_t, RlctMutexAttr);
assert_equal_size!(pthread_cond_t, RlctCond);
assert_equal_size!(pthread_condattr_t, RlctCondAttr);
assert_equal_size!(pthread_spinlock_t, RlctSpinlock);
assert_equal_size!(pthread_once_t, RlctOnce);

pub type pthread_t = *mut c_void;
pub type pthread_key_t = c_ulong;
