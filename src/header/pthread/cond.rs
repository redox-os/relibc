// Used design from https://www.remlab.net/op/futex-condvar.shtml

use super::*;

// PTHREAD_COND_INITIALIZER is defined manually in bits_pthread/cbindgen.toml

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> c_int {
    let cond = unsafe { &*cond.cast::<RlctCond>() };
    e(cond.broadcast())
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_destroy(cond: *mut pthread_cond_t) -> c_int {
    // No-op
    unsafe { core::ptr::drop_in_place(cond.cast::<RlctCond>()) };
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_init(
    cond: *mut pthread_cond_t,
    _attr: *const pthread_condattr_t,
) -> c_int {
    let cond_value = RlctCond::new();
    unsafe { cond.cast::<RlctCond>().write(cond_value) };

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_signal(cond: *mut pthread_cond_t) -> c_int {
    let cond = unsafe { &*cond.cast::<RlctCond>() };
    e(cond.signal())
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_timedwait(
    cond: *mut pthread_cond_t,
    mutex: *mut pthread_mutex_t,
    timeout: *const timespec,
) -> c_int {
    let cond = unsafe { &*cond.cast::<RlctCond>() };
    let mutex = unsafe { &*mutex.cast::<RlctMutex>() };
    let timeout = unsafe { &*timeout };
    e(cond.timedwait(mutex, timeout))
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_wait(
    cond: *mut pthread_cond_t,
    mutex: *mut pthread_mutex_t,
) -> c_int {
    let cond = unsafe { &*cond.cast::<RlctCond>() };
    let mutex = unsafe { &*mutex.cast::<RlctMutex>() };
    e(cond.wait(mutex))
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_destroy(condattr: *mut pthread_condattr_t) -> c_int {
    unsafe { core::ptr::drop_in_place(condattr.cast::<RlctCondAttr>()) };
    // No-op
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_getclock(
    condattr: *const pthread_condattr_t,
    clock: *mut clockid_t,
) -> c_int {
    let condattr = unsafe { &*condattr.cast::<RlctCondAttr>() };
    unsafe { core::ptr::write(clock, condattr.clock) };
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_getpshared(
    condattr: *const pthread_condattr_t,
    pshared: *mut c_int,
) -> c_int {
    let condattr = unsafe { &*condattr.cast::<RlctCondAttr>() };
    unsafe { core::ptr::write(pshared, condattr.pshared) };
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_init(condattr: *mut pthread_condattr_t) -> c_int {
    let condattr_value = RlctCondAttr::default();
    unsafe { condattr.cast::<RlctCondAttr>().write(condattr_value) };

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_setclock(
    condattr: *mut pthread_condattr_t,
    clock: clockid_t,
) -> c_int {
    let condattr = unsafe { &mut *condattr.cast::<RlctCondAttr>() };
    condattr.clock = clock;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_setpshared(
    condattr: *mut pthread_condattr_t,
    pshared: c_int,
) -> c_int {
    let condattr = unsafe { &mut *condattr.cast::<RlctCondAttr>() };
    condattr.pshared = pshared;
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
