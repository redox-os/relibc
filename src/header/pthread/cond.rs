// Used design from https://www.remlab.net/op/futex-condvar.shtml

use super::*;

use core::sync::atomic::{AtomicI32 as AtomicInt, Ordering};

// PTHREAD_COND_INITIALIZER

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> c_int {
    wake(cond, i32::MAX)
}

unsafe fn wake(cond: *mut pthread_cond_t, n: i32) -> c_int {
    let cond = &*cond.cast::<RlctCond>();

    // This is formally correct as long as we don't have more than u32::MAX threads.
    let prev = cond.prev.load(Ordering::SeqCst);
    cond.cur.store(prev.wrapping_add(1), Ordering::SeqCst);

    crate::sync::futex_wake(&cond.cur, n);

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_destroy(cond: *mut pthread_cond_t) -> c_int {
    let _cond = &mut cond.cast::<RlctCond>();

    // No-op
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_init(cond: *mut pthread_cond_t, _attr: *const pthread_condattr_t) -> c_int {
    cond.cast::<RlctCond>().write(RlctCond {
        cur: AtomicInt::new(0),
        prev: AtomicInt::new(0),
    });
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_signal(cond: *mut pthread_cond_t) -> c_int {
    wake(cond, 1)
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_timedwait(cond: *mut pthread_cond_t, mutex_ptr: *mut pthread_mutex_t, timeout: *const timespec) -> c_int {
    // TODO: Error checking for certain types (i.e. robust and errorcheck) of mutexes, e.g. if the
    // mutex is not locked.
    let cond = &*cond.cast::<RlctCond>();
    let timeout: Option<&timespec> = timeout.as_ref();

    let current = cond.cur.load(Ordering::Relaxed);
    cond.prev.store(current, Ordering::SeqCst); // TODO: ordering?

    pthread_mutex_unlock(mutex_ptr);
    crate::sync::futex_wait(&cond.cur, current, timeout);
    pthread_mutex_lock(mutex_ptr);

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_wait(cond: *mut pthread_cond_t, mutex: *mut pthread_mutex_t) -> c_int {
    pthread_cond_timedwait(cond, mutex, core::ptr::null())
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_destroy(condattr: *mut pthread_condattr_t) -> c_int {
    let _condattr = &mut *condattr.cast::<RlctCondAttr>();

    // No-op
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_getclock(condattr: *const pthread_condattr_t, clock: *mut clockid_t) -> c_int {
    core::ptr::write(clock, (*condattr.cast::<RlctCondAttr>()).clock);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_getpshared(condattr: *const pthread_condattr_t, pshared: *mut c_int) -> c_int {
    core::ptr::write(pshared, (*condattr.cast::<RlctCondAttr>()).pshared);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_init(condattr: *mut pthread_condattr_t) -> c_int {
    condattr.cast::<RlctCondAttr>().write(RlctCondAttr {
        // FIXME: system clock
        clock: 0,
        // Default
        pshared: PTHREAD_PROCESS_PRIVATE,
    });

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_setclock(condattr: *mut pthread_condattr_t, clock: clockid_t) -> c_int {
    (*condattr.cast::<RlctCondAttr>()).clock = clock;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_setpshared(condattr: *mut pthread_condattr_t, pshared: c_int) -> c_int {
    (*condattr.cast::<RlctCondAttr>()).pshared = pshared;
    0
}

pub(crate) struct RlctCondAttr {
    pub clock: clockid_t,
    pub pshared: c_int,
}

pub(crate) struct RlctCond {
    pub cur: AtomicInt,
    pub prev: AtomicInt,
}
