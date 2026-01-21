// Used design from https://www.remlab.net/op/futex-condvar.shtml

use crate::header::time::CLOCK_REALTIME;

use super::*;

// PTHREAD_COND_INITIALIZER is defined manually in bits_pthread/cbindgen.toml

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> c_int {
    e((unsafe { &*cond.cast::<RlctCond>() }).broadcast())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_cond_destroy(cond: *mut pthread_cond_t) -> c_int {
    // No-op
    unsafe { core::ptr::drop_in_place(cond.cast::<RlctCond>()) };
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_cond_init(
    cond: *mut pthread_cond_t,
    attr: *const pthread_condattr_t,
) -> c_int {
    let attr = unsafe { attr.cast::<RlctCondAttr>().as_ref() }
        .copied()
        .unwrap_or_default();

    if attr.clock != CLOCK_REALTIME {
        // As monotonic clock always smaller than realtime clock, this always result in instant timeout.
        println!("TODO: pthread_cond_init with monotonic clock");
    }

    unsafe { cond.cast::<RlctCond>().write(RlctCond::new()) };

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_cond_signal(cond: *mut pthread_cond_t) -> c_int {
    e((unsafe { &*cond.cast::<RlctCond>() }).signal())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_cond_timedwait(
    cond: *mut pthread_cond_t,
    mutex: *mut pthread_mutex_t,
    timeout: *const timespec,
) -> c_int {
    e((unsafe { &*cond.cast::<RlctCond>() })
        .timedwait(unsafe { &*mutex.cast::<RlctMutex>() }, unsafe { &*timeout }))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_cond_clockwait(
    cond: *mut pthread_cond_t,
    mutex: *mut pthread_mutex_t,
    clock_id: clockid_t,
    timeout: *const timespec,
) -> c_int {
    e((unsafe { &*cond.cast::<RlctCond>() }).clockwait(
        unsafe { &*mutex.cast::<RlctMutex>() },
        unsafe { &*timeout },
        clock_id,
    ))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_cond_wait(
    cond: *mut pthread_cond_t,
    mutex: *mut pthread_mutex_t,
) -> c_int {
    e((unsafe { &*cond.cast::<RlctCond>() }).wait(unsafe { &*mutex.cast::<RlctMutex>() }))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_condattr_destroy(condattr: *mut pthread_condattr_t) -> c_int {
    unsafe { core::ptr::drop_in_place(condattr.cast::<RlctCondAttr>()) };
    // No-op
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_condattr_getclock(
    condattr: *const pthread_condattr_t,
    clock: *mut clockid_t,
) -> c_int {
    unsafe { core::ptr::write(clock, (*condattr.cast::<RlctCondAttr>()).clock) };
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_condattr_getpshared(
    condattr: *const pthread_condattr_t,
    pshared: *mut c_int,
) -> c_int {
    unsafe { core::ptr::write(pshared, (*condattr.cast::<RlctCondAttr>()).pshared) };
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_condattr_init(condattr: *mut pthread_condattr_t) -> c_int {
    unsafe {
        condattr
            .cast::<RlctCondAttr>()
            .write(RlctCondAttr::default())
    };
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_condattr_setclock(
    condattr: *mut pthread_condattr_t,
    clock: clockid_t,
) -> c_int {
    (unsafe { *condattr.cast::<RlctCondAttr>() }).clock = clock;
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_condattr_setpshared(
    condattr: *mut pthread_condattr_t,
    pshared: c_int,
) -> c_int {
    (unsafe { *condattr.cast::<RlctCondAttr>() }).pshared = pshared;
    0
}

pub(crate) type RlctCondAttr = crate::sync::cond::CondAttr;

pub(crate) type RlctCond = crate::sync::cond::Cond;
