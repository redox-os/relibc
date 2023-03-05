use super::*;

use crate::header::errno::EBUSY;

// PTHREAD_MUTEX_INITIALIZER

#[repr(C)]
pub struct Mutex {
    pub(crate) inner: crate::sync::Mutex<()>,
}

#[repr(C)]
pub struct MutexAttr {
    prioceiling: c_int,
    protocol: c_int,
    pshared: c_int,
    robust: c_int,
    ty: c_int,
}

// #[no_mangle]
pub extern "C" fn pthread_mutex_consistent(mutex: *mut pthread_mutex_t) -> c_int {
    todo!();
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_destroy(mutex: *mut pthread_mutex_t) -> c_int {
    let _mutex: &pthread_mutex_t = &*mutex;
    0
}

// #[no_mangle]
pub extern "C" fn pthread_mutex_getprioceiling(mutex: *const pthread_mutex_t, prioceiling: *mut c_int) -> c_int {
    todo!();
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_init(mutex: *mut pthread_mutex_t, _attr: *const pthread_mutexattr_t) -> c_int {
    // TODO: attr
    mutex.write(Mutex {
        inner: crate::sync::Mutex::new(()),
    });
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_lock(mutex: *mut pthread_mutex_t) -> c_int {
    (&*mutex).inner.manual_lock();

    0
}

// #[no_mangle]
pub extern "C" fn pthread_mutex_setprioceiling(mutex: *mut pthread_mutex_t, prioceiling: c_int, old_prioceiling: *mut c_int) -> c_int {
    todo!();
}

// #[no_mangle]
pub extern "C" fn pthread_mutex_timedlock(mutex: *mut pthread_mutex_t, timespec: *const timespec) -> c_int {
    todo!();
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_trylock(mutex: *mut pthread_mutex_t) -> c_int {
    match (&*mutex).inner.manual_try_lock() {
        Ok(_) => 0,
        Err(_) => EBUSY,
    }
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_unlock(mutex: *mut pthread_mutex_t) -> c_int {
    (&*mutex).inner.manual_unlock();
    0
}

#[no_mangle]
pub extern "C" fn pthread_mutexattr_destroy(_attr: *mut pthread_mutexattr_t) -> c_int {
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_getprioceiling(attr: *const pthread_mutexattr_t, prioceiling: *mut c_int) -> c_int {
    prioceiling.write((*attr).prioceiling);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_getprotocol(attr: *const pthread_mutexattr_t, protocol: *mut c_int) -> c_int {
    protocol.write((*attr).protocol);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_getpshared(attr: *const pthread_mutexattr_t, pshared: *mut c_int) -> c_int {
    pshared.write((*attr).pshared);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_getrobust(attr: *const pthread_mutexattr_t, robust: *mut c_int) -> c_int {
    robust.write((*attr).robust);
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_gettype(attr: *const pthread_mutexattr_t, ty: *mut c_int) -> c_int {
    ty.write((*attr).ty);
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_init(attr: *mut pthread_mutexattr_t) -> c_int {
    attr.write(MutexAttr {
        robust: PTHREAD_MUTEX_STALLED,
        pshared: PTHREAD_PROCESS_PRIVATE,
        protocol: PTHREAD_PRIO_NONE,
        // TODO
        prioceiling: 0,
        ty: PTHREAD_MUTEX_DEFAULT,
    });
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_setprioceiling(attr: *mut pthread_mutexattr_t, prioceiling: c_int) -> c_int {
    (*attr).prioceiling = prioceiling;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_setprotocol(attr: *mut pthread_mutexattr_t, protocol: c_int) -> c_int {
    (*attr).protocol = protocol;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_setpshared(attr: *mut pthread_mutexattr_t, pshared: c_int) -> c_int {
    (*attr).pshared = pshared;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_setrobust(attr: *mut pthread_mutexattr_t, robust: c_int) -> c_int {
    (*attr).robust = robust;
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_mutexattr_settype(attr: *mut pthread_mutexattr_t, ty: c_int) -> c_int {
    (*attr).ty = ty;
    0
}
