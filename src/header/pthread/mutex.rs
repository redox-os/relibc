use super::*;

use crate::{
    error::Errno,
    header::{
        errno::{EINVAL, ENOMEM, ETIMEDOUT},
        time::{CLOCK_MONOTONIC, CLOCK_REALTIME, timespec_realtime_to_monotonic},
    },
};

// PTHREAD_MUTEX_INITIALIZER is defined in bits_pthread/cbindgen.toml

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_consistent(mutex: *mut pthread_mutex_t) -> c_int {
    e((unsafe { &*mutex.cast::<RlctMutex>() }).make_consistent())
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_destroy(mutex: *mut pthread_mutex_t) -> c_int {
    // No-op
    unsafe { core::ptr::drop_in_place(mutex.cast::<RlctMutex>()) };
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_getprioceiling(
    mutex: *const pthread_mutex_t,
    prioceiling: *mut c_int,
) -> c_int {
    match (unsafe { &*mutex.cast::<RlctMutex>() }).prioceiling() {
        Ok(value) => {
            unsafe { prioceiling.write(value) };
            0
        }
        Err(Errno(errno)) => errno,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_init(
    mutex: *mut pthread_mutex_t,
    attr: *const pthread_mutexattr_t,
) -> c_int {
    let attr = unsafe { attr.cast::<RlctMutexAttr>().as_ref() }
        .copied()
        .unwrap_or_default();

    match RlctMutex::new(&attr) {
        Ok(new) => {
            unsafe { mutex.cast::<RlctMutex>().write(new) };

            0
        }
        Err(Errno(errno)) => errno,
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_lock(mutex: *mut pthread_mutex_t) -> c_int {
    e((unsafe { &*mutex.cast::<RlctMutex>() }).lock())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_setprioceiling(
    mutex: *mut pthread_mutex_t,
    prioceiling: c_int,
    old_prioceiling: *mut c_int,
) -> c_int {
    match (unsafe { &*mutex.cast::<RlctMutex>() }).replace_prioceiling(prioceiling) {
        Ok(old) => {
            unsafe { old_prioceiling.write(old) };
            0
        }
        Err(Errno(errno)) => errno,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_timedlock(
    mutex: *mut pthread_mutex_t,
    abstime: *const timespec,
) -> c_int {
    let relative = match timespec_realtime_to_monotonic(unsafe { *abstime }) {
        Ok(relative) => relative,
        Err(err) => return e(Err(err)),
    };

    e((unsafe { &*mutex.cast::<RlctMutex>() }).lock_with_timeout(&relative))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_trylock(mutex: *mut pthread_mutex_t) -> c_int {
    e((unsafe { &*mutex.cast::<RlctMutex>() }).try_lock())
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_unlock(mutex: *mut pthread_mutex_t) -> c_int {
    e((unsafe { &*mutex.cast::<RlctMutex>() }).unlock())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_destroy(attr: *mut pthread_mutexattr_t) -> c_int {
    // No-op
    unsafe { core::ptr::drop_in_place(attr.cast::<RlctMutexAttr>()) };
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_getprioceiling(
    attr: *const pthread_mutexattr_t,
    prioceiling: *mut c_int,
) -> c_int {
    unsafe { prioceiling.write((*attr.cast::<RlctMutexAttr>()).prioceiling) };
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_getprotocol(
    attr: *const pthread_mutexattr_t,
    protocol: *mut c_int,
) -> c_int {
    unsafe { protocol.write((*attr.cast::<RlctMutexAttr>()).protocol) };
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_getpshared(
    attr: *const pthread_mutexattr_t,
    pshared: *mut c_int,
) -> c_int {
    unsafe { pshared.write((*attr.cast::<RlctMutexAttr>()).pshared) };
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_getrobust(
    attr: *const pthread_mutexattr_t,
    robust: *mut c_int,
) -> c_int {
    unsafe { robust.write((*attr.cast::<RlctMutexAttr>()).robust) };
    0
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_gettype(
    attr: *const pthread_mutexattr_t,
    ty: *mut c_int,
) -> c_int {
    unsafe { ty.write((*attr.cast::<RlctMutexAttr>()).ty) };
    0
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_init(attr: *mut pthread_mutexattr_t) -> c_int {
    unsafe { attr.cast::<RlctMutexAttr>().write(RlctMutexAttr::default()) };
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_setprioceiling(
    attr: *mut pthread_mutexattr_t,
    prioceiling: c_int,
) -> c_int {
    (unsafe { *attr.cast::<RlctMutexAttr>() }).prioceiling = prioceiling;
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_setprotocol(
    attr: *mut pthread_mutexattr_t,
    protocol: c_int,
) -> c_int {
    (unsafe { *attr.cast::<RlctMutexAttr>() }).protocol = protocol;
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_setpshared(
    attr: *mut pthread_mutexattr_t,
    pshared: c_int,
) -> c_int {
    (unsafe { *attr.cast::<RlctMutexAttr>() }).pshared = pshared;
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_setrobust(
    attr: *mut pthread_mutexattr_t,
    robust: c_int,
) -> c_int {
    (unsafe { *attr.cast::<RlctMutexAttr>() }).robust = robust;
    0
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_settype(
    attr: *mut pthread_mutexattr_t,
    ty: c_int,
) -> c_int {
    (unsafe { *attr.cast::<RlctMutexAttr>() }).ty = ty;
    0
}

pub use crate::sync::pthread_mutex::RlctMutex;

#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct RlctMutexAttr {
    pub prioceiling: c_int,
    pub protocol: c_int,
    pub pshared: c_int,
    pub robust: c_int,
    pub ty: c_int,
}
impl Default for RlctMutexAttr {
    fn default() -> Self {
        Self {
            robust: PTHREAD_MUTEX_STALLED,
            pshared: PTHREAD_PROCESS_PRIVATE,
            protocol: PTHREAD_PRIO_NONE,
            // TODO
            prioceiling: 0,
            ty: PTHREAD_MUTEX_DEFAULT,
        }
    }
}
