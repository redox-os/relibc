// FIXME(andypython): remove this when #![allow(warnings, unused_variables)] is
// dropped from src/lib.rs.
#![warn(warnings, unused_variables)]

use super::*;
pub use crate::sync::pthread_mutex::RlctMutex;
use crate::{error::Errno, header::time::timespec_realtime_to_monotonic};

// PTHREAD_MUTEX_INITIALIZER is defined in bits_pthread/cbindgen.toml

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_mutex_consistent.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_consistent(mutex: *mut pthread_mutex_t) -> c_int {
    e((unsafe { &*mutex.cast::<RlctMutex>() }).make_consistent())
}
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_mutex_destroy.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_destroy(mutex: *mut pthread_mutex_t) -> c_int {
    // No-op
    unsafe { core::ptr::drop_in_place(mutex.cast::<RlctMutex>()) };
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_mutex_getprioceiling.html>.
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

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_mutex_init.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_init(
    mutex: *mut pthread_mutex_t,
    attr: *const pthread_mutexattr_t,
) -> c_int {
    let attr = unsafe { attr.cast::<RlctMutexAttr>().as_ref() }
        .cloned()
        .unwrap_or_default();

    match RlctMutex::new(&attr) {
        Ok(new) => {
            unsafe { mutex.cast::<RlctMutex>().write(new) };

            0
        }
        Err(Errno(errno)) => errno,
    }
}
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_mutex_lock.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_lock(mutex: *mut pthread_mutex_t) -> c_int {
    e((unsafe { &*mutex.cast::<RlctMutex>() }).lock())
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_mutex_setprioceiling.html>.
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

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_mutex_timedlock.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_timedlock(
    mutex: *mut pthread_mutex_t,
    abstime: &timespec,
) -> c_int {
    let relative = match timespec_realtime_to_monotonic(abstime) {
        Ok(relative) => relative,
        Err(err) => return e(Err(err)),
    };

    e((unsafe { &*mutex.cast::<RlctMutex>() }).lock_with_timeout(&relative))
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_mutex_trylock.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_trylock(mutex: *mut pthread_mutex_t) -> c_int {
    e((unsafe { &*mutex.cast::<RlctMutex>() }).try_lock())
}
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_mutex_unlock.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutex_unlock(mutex: *mut pthread_mutex_t) -> c_int {
    e((unsafe { &*mutex.cast::<RlctMutex>() }).unlock())
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_mutexattr_destroy.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_destroy(attr: *mut pthread_mutexattr_t) -> c_int {
    // No-op
    unsafe { core::ptr::drop_in_place(attr.cast::<RlctMutexAttr>()) };
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_mutexattr_getprioceiling.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_getprioceiling(
    attr: *const pthread_mutexattr_t,
    prioceiling: &mut c_int,
) -> c_int {
    *prioceiling = unsafe { &*attr.cast::<RlctMutexAttr>() }.prioceiling;
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_mutexattr_getprotocol.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_getprotocol(
    attr: *const pthread_mutexattr_t,
    protocol: &mut c_int,
) -> c_int {
    *protocol = unsafe { &*attr.cast::<RlctMutexAttr>() }.protocol;
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_mutexattr_getpshared.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_getpshared(
    attr: *const pthread_mutexattr_t,
    pshared: &mut c_int,
) -> c_int {
    *pshared = unsafe { &*attr.cast::<RlctMutexAttr>() }.pshared;
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_mutexattr_getrobust.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_getrobust(
    attr: *const pthread_mutexattr_t,
    robust: &mut c_int,
) -> c_int {
    *robust = unsafe { &*attr.cast::<RlctMutexAttr>() }.robust;
    0
}
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_mutexattr_gettype.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_gettype(
    attr: *const pthread_mutexattr_t,
    ty: &mut c_int,
) -> c_int {
    *ty = unsafe { &*attr.cast::<RlctMutexAttr>() }.ty;
    0
}
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_mutexattr_init.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_init(attr: *mut pthread_mutexattr_t) -> c_int {
    unsafe { attr.cast::<RlctMutexAttr>().write(RlctMutexAttr::default()) };
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_mutexattr_setprioceiling.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_setprioceiling(
    attr: *mut pthread_mutexattr_t,
    prioceiling: c_int,
) -> c_int {
    unsafe { &mut *attr.cast::<RlctMutexAttr>() }.prioceiling = prioceiling;
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_mutexattr_setprotocol.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_setprotocol(
    attr: *mut pthread_mutexattr_t,
    protocol: c_int,
) -> c_int {
    unsafe { &mut *attr.cast::<RlctMutexAttr>() }.protocol = protocol;
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_mutexattr_setpshared.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_setpshared(
    attr: *mut pthread_mutexattr_t,
    pshared: c_int,
) -> c_int {
    unsafe { &mut *attr.cast::<RlctMutexAttr>() }.pshared = pshared;
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_mutexattr_setrobust.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_setrobust(
    attr: *mut pthread_mutexattr_t,
    robust: c_int,
) -> c_int {
    unsafe { &mut *attr.cast::<RlctMutexAttr>() }.robust = robust;
    0
}
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_mutexattr_settype.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_mutexattr_settype(
    attr: *mut pthread_mutexattr_t,
    ty: c_int,
) -> c_int {
    unsafe { &mut *attr.cast::<RlctMutexAttr>() }.ty = ty;
    0
}

#[repr(C)]
#[derive(Clone)]
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
