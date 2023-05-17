// Used design from https://www.remlab.net/op/futex-condvar.shtml

use super::*;

// PTHREAD_COND_INITIALIZER is defined manually in bits_pthread/cbindgen.toml

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> c_int {
    e((&*cond.cast::<RlctCond>()).broadcast())
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_destroy(cond: *mut pthread_cond_t) -> c_int {
    // No-op
    core::ptr::drop_in_place(cond.cast::<RlctCond>());
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_init(
    cond: *mut pthread_cond_t,
    _attr: *const pthread_condattr_t,
) -> c_int {
    cond.cast::<RlctCond>().write(RlctCond::new());

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_signal(cond: *mut pthread_cond_t) -> c_int {
    e((&*cond.cast::<RlctCond>()).signal())
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_timedwait(
    cond: *mut pthread_cond_t,
    mutex: *mut pthread_mutex_t,
    timeout: *const timespec,
) -> c_int {
    e((&*cond.cast::<RlctCond>()).timedwait(&*mutex.cast::<RlctMutex>(), &*timeout))
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_wait(
    cond: *mut pthread_cond_t,
    mutex: *mut pthread_mutex_t,
) -> c_int {
    e((&*cond.cast::<RlctCond>()).wait(&*mutex.cast::<RlctMutex>()))
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_destroy(condattr: *mut pthread_condattr_t) -> c_int {
    core::ptr::drop_in_place(condattr.cast::<RlctCondAttr>());
    // No-op
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_getclock(
    condattr: *const pthread_condattr_t,
    clock: *mut clockid_t,
) -> c_int {
    core::ptr::write(clock, (*condattr.cast::<RlctCondAttr>()).clock);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_getpshared(
    condattr: *const pthread_condattr_t,
    pshared: *mut c_int,
) -> c_int {
    core::ptr::write(pshared, (*condattr.cast::<RlctCondAttr>()).pshared);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_init(condattr: *mut pthread_condattr_t) -> c_int {
    condattr
        .cast::<RlctCondAttr>()
        .write(RlctCondAttr::default());

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_setclock(
    condattr: *mut pthread_condattr_t,
    clock: clockid_t,
) -> c_int {
    (*condattr.cast::<RlctCondAttr>()).clock = clock;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_setpshared(
    condattr: *mut pthread_condattr_t,
    pshared: c_int,
) -> c_int {
    (*condattr.cast::<RlctCondAttr>()).pshared = pshared;
    0
}

pub(crate) struct RlctCondAttr {
    clock: clockid_t,
    pshared: c_int,
}

pub(crate) type RlctCond = crate::sync::cond::Cond;

impl Default for RlctCondAttr {
    fn default() -> Self {
        Self {
            // FIXME: system clock
            clock: 0,
            // Default
            pshared: PTHREAD_PROCESS_PRIVATE,
        }
    }
}
