// Used design from https://www.remlab.net/op/futex-condvar.shtml

use super::*;

use core::sync::atomic::{AtomicI32 as AtomicInt, Ordering};

// PTHREAD_COND_INITIALIZER

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> c_int {
    wake(cond, i32::MAX)
}

unsafe fn wake(cond: *mut pthread_cond_t, n: i32) -> c_int {
    let cond: &pthread_cond_t = &*cond;

    // This is formally correct as long as we don't have more than u32::MAX threads.
    let prev = cond.prev.load(Ordering::SeqCst);
    cond.cur.store(prev.wrapping_add(1), Ordering::SeqCst);

    crate::sync::futex_wake(&cond.cur, n);

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_destroy(cond: *mut pthread_cond_t) -> c_int {
    let _cond: &pthread_cond_t = &*cond;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_cond_init(cond: *mut pthread_cond_t, _attr: *const pthread_condattr_t) -> c_int {
    cond.write(pthread_cond_t {
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
    let cond: &pthread_cond_t = &*cond;
    let timeout: Option<&timespec> = timeout.as_ref();

    let current = cond.cur.load(Ordering::Relaxed);
    cond.prev.store(current, Ordering::SeqCst);

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
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_getclock(condattr: *const pthread_condattr_t, clock: *mut clockid_t) -> c_int {
    core::ptr::write(clock, (*condattr).clock);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_getpshared(condattr: *const pthread_condattr_t, pshared: *mut c_int) -> c_int {
    core::ptr::write(pshared, (*condattr).pshared);
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_init(condattr: *mut pthread_condattr_t) -> c_int {
    core::ptr::write(condattr, pthread_condattr_t {
        // FIXME: system clock
        clock: 0,
        // Default
        pshared: PTHREAD_PROCESS_PRIVATE,
    });
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_setclock(condattr: *mut pthread_condattr_t, clock: clockid_t) -> c_int {
    (*condattr).clock = clock;
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_condattr_setpshared(condattr: *mut pthread_condattr_t, pshared: c_int) -> c_int {
    (*condattr).pshared = pshared;
    0
}

