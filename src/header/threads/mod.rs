//! `threads.h` implementation.
//! Based on the musl implementation, with regard for our special implementation of pthread types
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/threads.h.html>.
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
use super::{
    bits_pthread::{self, pthread_cond_t, pthread_key_t, pthread_mutex_t, pthread_mutexattr_t},
    errno::{EBUSY, EINTR, ENOMEM, ETIMEDOUT},
    limits,
    pthread::{
        self, PTHREAD_MUTEX_NORMAL, PTHREAD_MUTEX_RECURSIVE, RlctCond, pthread_cond_broadcast,
        pthread_cond_destroy, pthread_cond_signal, pthread_cond_timedwait, pthread_cond_wait,
        pthread_create, pthread_detach, pthread_equal, pthread_exit, pthread_getspecific,
        pthread_join, pthread_key_create, pthread_key_delete, pthread_mutex_destroy,
        pthread_mutex_init, pthread_mutex_lock, pthread_mutex_timedlock, pthread_mutex_trylock,
        pthread_mutex_unlock, pthread_mutexattr_init, pthread_mutexattr_settype, pthread_self,
        pthread_setspecific,
    },
    sched::sched_yield,
    time::{CLOCK_REALTIME, clock_nanosleep, timespec},
};
use crate::platform::{
    ERRNO,
    types::{c_int, c_long, c_void},
};
use core::mem::MaybeUninit;

pub type thrd_t = bits_pthread::pthread_t;
pub type once_flag = bits_pthread::pthread_once_t;
pub type tss_t = bits_pthread::pthread_key_t;
pub type mtx_t = bits_pthread::pthread_mutex_t;
pub type cnd_t = bits_pthread::pthread_cond_t;

pub type thrd_start_t = extern "C" fn(*mut c_void) -> c_int;
pub type tss_dtor_t = Option<extern "C" fn(*mut c_void)>;

pub const thrd_success: c_int = 0;
pub const thrd_busy: c_int = 1;
pub const thrd_error: c_int = 2;
pub const thrd_nomem: c_int = 3;
pub const thrd_timedout: c_int = 4;

pub const mtx_plain: c_int = 0;
pub const mtx_recursive: c_int = 1;
pub const mtx_timed: c_int = 2;

pub const TSS_DTOR_ITERATIONS: c_long = limits::PTHREAD_DESTRUCTOR_ITERATIONS;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/call_once.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn call_once(flag: *mut once_flag, func: extern "C" fn()) {
    unsafe {
        pthread::pthread_once(flag, func);
    };
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cnd_broadcast.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cnd_broadcast(cnd: *mut cnd_t) -> c_int {
    if unsafe { pthread_cond_broadcast(cnd.cast::<pthread_cond_t>()) } == 0 {
        thrd_success
    } else {
        thrd_error
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cnd_destroy.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cnd_destroy(cnd: *mut cnd_t) {
    unsafe { pthread_cond_destroy(cnd.cast::<pthread_cond_t>()) };
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cnd_init.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cnd_init(cnd: *mut cnd_t) -> c_int {
    unsafe {
        cnd.cast::<RlctCond>().write(RlctCond::new());
    }
    thrd_success
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cnd_signal.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cnd_signal(cnd: *mut cnd_t) -> c_int {
    if unsafe { pthread_cond_signal(cnd.cast::<pthread_cond_t>()) } == 0 {
        thrd_success
    } else {
        thrd_error
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cnd_timedwait.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cnd_timedwait(
    cnd: *mut cnd_t,
    mtx: *mut mtx_t,
    timeout: *const timespec,
) -> c_int {
    ERRNO.set(0);
    let ret = unsafe {
        pthread_cond_timedwait(
            cnd.cast::<pthread_cond_t>(),
            mtx.cast::<pthread_mutex_t>(),
            timeout,
        )
    };
    let new_errno = ERRNO.get();
    match ret {
        0 => thrd_success,
        _ if new_errno == ETIMEDOUT => thrd_timedout,
        _ => thrd_error,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/cnd_wait.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cnd_wait(cnd: *mut cnd_t, mtx: *mut mtx_t) -> c_int {
    if unsafe { pthread_cond_wait(cnd.cast::<pthread_cond_t>(), mtx.cast::<pthread_mutex_t>()) }
        == 0
    {
        thrd_success
    } else {
        thrd_error
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mtx_destroy.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mtx_destroy(mtx: *mut mtx_t) {
    unsafe {
        pthread_mutex_destroy(mtx.cast::<pthread_mutex_t>());
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mtx_init.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mtx_init(mtx: *mut mtx_t, ty: c_int) -> c_int {
    let mut attr = MaybeUninit::<pthread_mutexattr_t>::uninit();
    let _ = unsafe {
        pthread_mutexattr_init(attr.as_mut_ptr());
    };
    let mut attr = unsafe { attr.assume_init() };

    let _ = unsafe {
        pthread_mutexattr_settype(
            &raw mut attr,
            if ty & mtx_recursive == mtx_recursive {
                PTHREAD_MUTEX_RECURSIVE
            } else {
                PTHREAD_MUTEX_NORMAL
            },
        )
    };
    if unsafe { pthread_mutex_init(mtx.cast::<pthread_mutex_t>(), &raw const attr) } == 0 {
        thrd_success
    } else {
        thrd_error
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mtx_lock.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mtx_lock(mtx: *mut mtx_t) -> c_int {
    if unsafe { pthread_mutex_lock(mtx.cast::<pthread_mutex_t>()) } == 0 {
        thrd_success
    } else {
        thrd_error
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mtx_timedlock.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mtx_timedlock(mtx: *mut mtx_t, timeout: &timespec) -> c_int {
    ERRNO.set(0);
    let ret = unsafe { pthread_mutex_timedlock(mtx.cast::<pthread_mutex_t>(), timeout) };
    let new_errno = ERRNO.get();
    match ret {
        0 => thrd_success,
        _ if new_errno == ETIMEDOUT => thrd_timedout,
        _ => thrd_error,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mtx_trylock.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mtx_trylock(mtx: *mut mtx_t) -> c_int {
    ERRNO.set(0);
    let ret = unsafe { pthread_mutex_trylock(mtx.cast::<pthread_mutex_t>()) };
    let new_errno = ERRNO.get();
    match ret {
        0 => thrd_success,
        _ if new_errno == EBUSY => thrd_busy,
        _ => thrd_error,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mtx_unlock.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mtx_unlock(mtx: *mut mtx_t) -> c_int {
    if unsafe { pthread_mutex_unlock(mtx.cast::<pthread_mutex_t>()) } == 0 {
        thrd_success
    } else {
        thrd_error
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/thrd_create.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn thrd_create(
    thrd: *mut thrd_t,
    start: thrd_start_t,
    arg: *mut c_void,
) -> c_int {
    ERRNO.set(0);
    let ret = unsafe {
        pthread_create(
            thrd,
            core::ptr::null(),
            core::mem::transmute::<_, extern "C" fn(*mut c_void) -> *mut c_void>(start),
            arg,
        )
    };
    let new_errno = ERRNO.get();
    match ret {
        0 => thrd_success,
        _ if new_errno == ENOMEM => thrd_nomem,
        _ => thrd_error,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/thrd_current.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn thrd_current() -> thrd_t {
    unsafe { pthread_self() }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/thrd_detach.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn thrd_detach(thrd: thrd_t) -> c_int {
    if unsafe { pthread_detach(thrd) } == 0 {
        thrd_success
    } else {
        thrd_error
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/thrd_equal.html>.
#[unsafe(no_mangle)]
pub extern "C" fn thrd_equal(thrd1: thrd_t, thrd2: thrd_t) -> c_int {
    pthread_equal(thrd1, thrd2)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/thrd_exit.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn thrd_exit(ret: c_int) -> ! {
    unsafe { pthread_exit(ret as *mut c_void) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/thrd_join.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn thrd_join(thrd: thrd_t, retval: *mut c_int) -> c_int {
    if unsafe { pthread_join(thrd, retval as *mut *mut c_void) } == 0 {
        thrd_success
    } else {
        thrd_error
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/thrd_sleep.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn thrd_sleep(time1: &timespec, time2: *mut timespec) -> c_int {
    ERRNO.set(0);
    let ret = clock_nanosleep(CLOCK_REALTIME, 0, time1, time2);
    let new_errno = ERRNO.get();
    match ret {
        0 => 0,
        -1 if new_errno == EINTR => -1,
        _ => -2,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/thrd_yield.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn thrd_yield() {
    sched_yield();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tss_create.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tss_create(tss: *mut tss_t, dtor: tss_dtor_t) -> c_int {
    if unsafe { pthread_key_create(tss.cast::<pthread_key_t>(), dtor) } == 0 {
        thrd_success
    } else {
        thrd_error
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tss_delete.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tss_delete(tss: tss_t) {
    // note: tss_t is a transparent type alias to pthread_key_t
    unsafe {
        pthread_key_delete(tss);
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tss_get.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tss_get(tss: tss_t) -> *mut c_void {
    // note: tss_t is a transparent type alias to pthread_key_t
    unsafe { pthread_getspecific(tss) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/tss_set.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tss_set(tss: tss_t, value: *mut c_void) -> c_int {
    // note: tss_t is a transparent type alias to pthread_key_t
    if unsafe { pthread_setspecific(tss, value) } == 0 {
        thrd_success
    } else {
        thrd_error
    }
}
