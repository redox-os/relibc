use super::*;

use crate::error::Errno;

// PTHREAD_MUTEX_INITIALIZER is defined in bits_pthread/cbindgen.toml

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_consistent(mutex: *mut pthread_mutex_t) -> c_int {
    let mutex = unsafe { &*mutex.cast::<RlctMutex>() };
    e(mutex.make_consistent())
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_destroy(mutex: *mut pthread_mutex_t) -> c_int {
    // No-op
    unsafe { core::ptr::drop_in_place(mutex.cast::<RlctMutex>()) };
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_getprioceiling(
    mutex: *const pthread_mutex_t,
    prioceiling: *mut c_int,
) -> c_int {
    let mutex = unsafe { &*mutex.cast::<RlctMutex>() };
    match mutex.prioceiling() {
        Ok(value) => {
            unsafe { prioceiling.write(value) };
            0
        }
        Err(Errno(errno)) => errno,
    }
}

#[no_mangle]
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
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_lock(mutex: *mut pthread_mutex_t) -> c_int {
    let mutex = unsafe { &*mutex.cast::<RlctMutex>() };
    e(mutex.lock())
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_setprioceiling(
    mutex: *mut pthread_mutex_t,
    prioceiling: c_int,
    old_prioceiling: *mut c_int,
) -> c_int {
    let mutex = unsafe { &*mutex.cast::<RlctMutex>() };
    match mutex.replace_prioceiling(prioceiling) {
        Ok(old) => {
            unsafe { old_prioceiling.write(old) };
            0
        }
        Err(Errno(errno)) => errno,
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_timedlock(
    mutex: *mut pthread_mutex_t,
    timespec: *const timespec,
) -> c_int {
    let mutex = unsafe { &*mutex.cast::<RlctMutex>() };
    let timespec = unsafe { &*timespec };
    e(mutex.lock_with_timeout(timespec))
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_trylock(mutex: *mut pthread_mutex_t) -> c_int {
    let mutex = unsafe { &*mutex.cast::<RlctMutex>() };
    e(mutex.try_lock())
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_unlock(mutex: *mut pthread_mutex_t) -> c_int {
    let mutex = unsafe { &*mutex.cast::<RlctMutex>() };
    e(mutex.unlock())
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_destroy(attr: *mut pthread_mutexattr_t) -> c_int {
    // No-op
    unsafe { core::ptr::drop_in_place(attr.cast::<RlctMutexAttr>()) };
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_getprioceiling(
    attr: *const pthread_mutexattr_t,
    prioceiling: *mut c_int,
) -> c_int {
    let attr = unsafe { &*attr.cast::<RlctMutexAttr>() };
    unsafe { prioceiling.write(attr.prioceiling) };
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_getprotocol(
    attr: *const pthread_mutexattr_t,
    protocol: *mut c_int,
) -> c_int {
    let attr = unsafe { &*attr.cast::<RlctMutexAttr>() };
    unsafe { protocol.write(attr.protocol) };
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_getpshared(
    attr: *const pthread_mutexattr_t,
    pshared: *mut c_int,
) -> c_int {
    let attr = unsafe { &*attr.cast::<RlctMutexAttr>() };
    unsafe { pshared.write(attr.pshared) };
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_getrobust(
    attr: *const pthread_mutexattr_t,
    robust: *mut c_int,
) -> c_int {
    let attr = unsafe { &*attr.cast::<RlctMutexAttr>() };
    unsafe { robust.write(attr.robust) };
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_gettype(
    attr: *const pthread_mutexattr_t,
    ty: *mut c_int,
) -> c_int {
    let attr = unsafe { &*attr.cast::<RlctMutexAttr>() };
    unsafe { ty.write(attr.ty) };
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_init(attr: *mut pthread_mutexattr_t) -> c_int {
    let attr_value = RlctMutexAttr::default();
    unsafe { attr.cast::<RlctMutexAttr>().write(attr_value) };
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_setprioceiling(
    attr: *mut pthread_mutexattr_t,
    prioceiling: c_int,
) -> c_int {
    let attr = unsafe { &mut *attr.cast::<RlctMutexAttr>() };
    attr.prioceiling = prioceiling;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_setprotocol(
    attr: *mut pthread_mutexattr_t,
    protocol: c_int,
) -> c_int {
    let attr = unsafe { &mut *attr.cast::<RlctMutexAttr>() };
    attr.protocol = protocol;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_setpshared(
    attr: *mut pthread_mutexattr_t,
    pshared: c_int,
) -> c_int {
    let attr = unsafe { &mut *attr.cast::<RlctMutexAttr>() };
    attr.pshared = pshared;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_setrobust(
    attr: *mut pthread_mutexattr_t,
    robust: c_int,
) -> c_int {
    let attr = unsafe { &mut *attr.cast::<RlctMutexAttr>() };
    attr.robust = robust;
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_settype(
    attr: *mut pthread_mutexattr_t,
    ty: c_int,
) -> c_int {
    let attr = unsafe { &mut *attr.cast::<RlctMutexAttr>() };
    attr.ty = ty;
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
