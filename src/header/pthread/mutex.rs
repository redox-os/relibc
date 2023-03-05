use super::*;

// PTHREAD_MUTEX_INITIALIZER

#[repr(C)]
pub struct Mutex {
}

#[repr(C)]
pub struct MutexAttr {
}

pub extern "C" fn pthread_mutex_consistent(mutex: *mut pthread_mutex_t) -> c_int {
    todo!();
}
pub extern "C" fn pthread_mutex_destroy(mutex: *mut pthread_mutex_t) -> c_int {
    todo!();
}

pub extern "C" fn pthread_mutex_getprioceiling(mutex: *const pthread_mutex_t, prioceiling: *mut c_int) -> c_int {
    todo!();
}

pub extern "C" fn pthread_mutex_init(mutex: *mut pthread_mutex_t, attr: *const pthread_mutexattr_t) -> c_int {
    todo!();
}
pub extern "C" fn pthread_mutex_lock(mutex: *mut pthread_mutex_t) -> c_int {
    todo!();
}

pub extern "C" fn pthread_mutex_setprioceiling(mutex: *mut pthread_mutex_t, prioceiling: c_int, old_prioceiling: *mut c_int) -> c_int {
    todo!();
}

pub extern "C" fn pthread_mutex_timedlock(mutex: *mut pthread_mutex_t, timespec: *const timespec) -> c_int {
    todo!();
}
pub extern "C" fn pthread_mutex_trylock(mutex: *mut pthread_mutex_t) -> c_int {
    todo!();
}
pub extern "C" fn pthread_mutex_unlock(mutex: *mut pthread_mutex_t) -> c_int {
    todo!();
}
pub extern "C" fn pthread_mutexattr_destroy(attr: *mut pthread_mutexattr_t) -> c_int {
    todo!();
}

pub extern "C" fn pthread_mutexattr_getprioceiling(attr: *const pthread_mutexattr_t, prioceiling: *mut c_int) -> c_int {
    todo!();
}


pub extern "C" fn pthread_mutexattr_getprotocol(attr: *const pthread_mutexattr_t, protocol: *mut c_int) -> c_int {
    todo!();
}

pub extern "C" fn pthread_mutexattr_getpshared(attr: *const pthread_mutexattr_t, pshared: *mut c_int) -> c_int {
    todo!();
}

pub extern "C" fn pthread_mutexattr_getrobust(attr: *const pthread_mutexattr_t, robust: *mut c_int) -> c_int {
    todo!();
}
pub extern "C" fn pthread_mutexattr_gettype(attr: *const pthread_mutexattr_t, ty: *mut c_int) -> c_int {
    todo!();
}
pub extern "C" fn pthread_mutexattr_init(attr: *mut pthread_mutexattr_t) -> c_int {
    todo!();
}

pub extern "C" fn pthread_mutexattr_setprioceiling(attr: *mut pthread_mutexattr_t, prioceiling: c_int) -> c_int {
    todo!();
}

pub extern "C" fn pthread_mutexattr_setprotocol(attr: *mut pthread_mutexattr_t, protocol: c_int) -> c_int {
    todo!();
}

pub extern "C" fn pthread_mutexattr_setpshared(attr: *mut pthread_mutexattr_t, pshared: c_int) -> c_int {
    todo!();
}

pub extern "C" fn pthread_mutexattr_setrobust(attr: *mut pthread_mutexattr_t, robust: c_int) -> c_int {
    todo!();
}
pub extern "C" fn pthread_mutexattr_settype(attr: *mut pthread_mutexattr_t, ty: c_int) -> c_int {
    todo!();
}
