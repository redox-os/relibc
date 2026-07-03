// Used design from https://www.remlab.net/op/futex-condvar.shtml

use crate::{
    header::{
        pthread::{PTHREAD_PROCESS_PRIVATE, PTHREAD_PROCESS_SHARED, RlctMutex, e},
        time::{CLOCK_REALTIME, timespec},
    },
    platform::types::{c_int, clockid_t, pthread_cond_t, pthread_condattr_t, pthread_mutex_t},
};

// PTHREAD_COND_INITIALIZER is defined manually in bits_pthread/cbindgen.toml

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_cond_broadcast.html>.
///
/// As a single atomic operation, determines which threads, if any, are blocked
/// on the specified condition variable `cond` and unblocks all of these
/// threads.
///
/// Has no effect if no threads are blocked on `cond`.
///
/// Upon success, returns `0`. Upon failure, an error number is returned to
/// indicate the error.
///
/// # Implementation
/// Always successful, so will never return an error number.
///
/// # Safety
/// It is undefined behaviour if `cond` is not initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_cond_broadcast(cond: *mut pthread_cond_t) -> c_int {
    e((unsafe { &*cond.cast::<RlctCond>() }).broadcast())
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_cond_destroy.html>.
///
/// Destroys the given condition variable specified by `cond`.
///
/// Upon success, returns `0`. Upon failure, an error number is returned to
/// indicate the error.
///
/// # Implementation
/// Always successful, so will never return an error number.
///
/// # Safety
/// It is undefined behaviour for the following:
/// - `cond` is not initialized when calling this function.
/// - Use of `cond` after calling this function without reinitializing it.
/// - Attempting to destroy a condition variable upon which other threads are
///   currently blocked.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_cond_destroy(cond: *mut pthread_cond_t) -> c_int {
    // No-op
    unsafe { core::ptr::drop_in_place(cond.cast::<RlctCond>()) };
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_cond_init.html>.
///
/// Initializes the condition variable referenced by `cond` with attributes
/// referenced by `attr`.
///
/// Upon success, returns `0`. Upon failure, an error number is returned to
/// indicate the error.
///
/// # Implementation
/// Always successful, so will never return an error number.
///
/// # Safety
/// It is undefined behaviour for the following:
/// - `attr` is not initialized when calling this function.
/// - `cond` has already been initialized.
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
        todo_skip!(0, "pthread_cond_init with monotonic clock");
    }

    unsafe { cond.cast::<RlctCond>().write(RlctCond::new()) };

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_cond_signal.html>.
///
/// As a single atomic operation, determines which threads, if any, are blocked
/// on the specified condition variable `cond` and unblocks at least one of
/// these threads.
///
/// Has no effect if no threads are blocked on `cond`.
///
/// Upon success, returns `0`. Upon failure, an error number is returned to
/// indicate the error.
///
/// # Implementation
/// Always successful, so will never return an error number.
/// Identical behaviour to `pthread_cond_broadcast()`.
///
/// # Safety
/// It is undefined behaviour if `cond` is not initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_cond_signal(cond: *mut pthread_cond_t) -> c_int {
    e((unsafe { &*cond.cast::<RlctCond>() }).signal())
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_cond_timedwait.html>.
///
/// Blocks on a condition variable.
///
/// Clock used for timeout depends on the clock attribute in `cond`. Defaults
/// to `CLOCK_REALTIME` if not set.
///
/// Upon success, returns `0` and the `mutex` shall have been locked and shall
/// be owned by the calling thread. Upon failure, an error number is returned
/// to indicate the error.
///
/// # Safety
/// It is undefined behaviour for the following:
/// - `cond` is uninitialized.
/// - `mutex` is uninitialized.
/// - `mutex` has not been locked by the calling thread before calling this
///   function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_cond_timedwait(
    cond: *mut pthread_cond_t,
    mutex: *mut pthread_mutex_t,
    abstime: *const timespec,
) -> c_int {
    e((unsafe { &*cond.cast::<RlctCond>() })
        .timedwait(unsafe { &*mutex.cast::<RlctMutex>() }, unsafe { &*abstime }))
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_cond_clockwait.html>.
///
/// Blocks on a condition variable.
///
/// `clock_id` determines the clock to use for the timeout specified by
/// `abstime`.
///
/// Upon success, returns `0` and the `mutex` shall have been locked and shall
/// be owned by the calling thread. Upon failure, an error number is returned
/// to indicate the error.
///
/// # Safety
/// It is undefined behaviour for the following:
/// - `cond` is uninitialized.
/// - `mutex` is uninitialized.
/// - `mutex` has not been locked by the calling thread before calling this
///   function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_cond_clockwait(
    cond: *mut pthread_cond_t,
    mutex: *mut pthread_mutex_t,
    clock_id: clockid_t,
    abstime: *const timespec,
) -> c_int {
    e((unsafe { &*cond.cast::<RlctCond>() }).clockwait(
        unsafe { &*mutex.cast::<RlctMutex>() },
        unsafe { &*abstime },
        clock_id,
    ))
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_cond_wait.html>.
///
/// Blocks on a condition variable.
///
/// Upon success, returns `0` and the `mutex` shall have been locked and shall
/// be owned by the calling thread. Upon failure, an error number is returned
/// to indicate the error.
///
/// # Safety
/// It is undefined behaviour for the following:
/// - `cond` is uninitialized.
/// - `mutex` is uninitialized.
/// - `mutex` has not been locked by the calling thread before calling this
///   function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_cond_wait(
    cond: *mut pthread_cond_t,
    mutex: *mut pthread_mutex_t,
) -> c_int {
    e((unsafe { &*cond.cast::<RlctCond>() }).wait(unsafe { &*mutex.cast::<RlctMutex>() }))
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_condattr_destroy.html>.
///
/// Destroys a condition variable attributes object.
///
/// Upon success, returns `0`. Upon failure, an error number is returned to
/// indicated the error.
///
/// # Implementation
/// Always successful, so will never return an error number.
///
/// # Safety
/// It is undefined behaviour for the following:
/// - `attr` is uninitialized.
/// - `attr` is used after calling this function without reinitializing it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_condattr_destroy(attr: *mut pthread_condattr_t) -> c_int {
    unsafe { core::ptr::drop_in_place(attr.cast::<RlctCondAttr>()) };
    // No-op
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_condattr_getclock.html>.
///
/// Obtains the value of the clock attribute from the attributes object
/// referenced by `attr`.
///
/// Upon success, returns `0` and stores the value of the clock attribute of
/// `attr` into the object refereced by the `clock_id` argument. Upon failure,
/// an error number is returned to indicate the error.
///
/// # Implementation
/// Always successful, so will never return an error number.
///
/// # Safety
/// It is undefined behaviour if `attr` is uninitialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_condattr_getclock(
    attr: *const pthread_condattr_t,
    clock_id: *mut clockid_t,
) -> c_int {
    unsafe { core::ptr::write(clock_id, (*attr.cast::<RlctCondAttr>()).clock) };
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_condattr_getpshared.html>.
///
/// Obtains the value of the process-shared attribute from the attributes
/// object referenced by `attr`.
///
/// Upon success, returns `0` and stores the value of the process-shared
/// attribute of `attr` into the object referenced by the `pshared` parameter.
/// Upon failure, an error number is returned to indicate the error.
///
/// # Implementation
/// Always successful, so will never return an error number.
///
/// # Safety
/// It is undefined behaviour if `attr` is uninitialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_condattr_getpshared(
    attr: *const pthread_condattr_t,
    pshared: *mut c_int,
) -> c_int {
    unsafe { core::ptr::write(pshared, (*attr.cast::<RlctCondAttr>()).pshared) };
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_condattr_init.html>.
///
/// Initializes a condition variable attribute object `attr` with the default
/// value for all of the attributes defined by the implementation.
///
/// Upon success, returns `0`. Upon failure, an error number is returned to
/// indicate the error.
///
/// # Implementation
/// Always successful, so will never return an error number.
///
/// # Safety
/// It is undefined behaviour if `attr` is already initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_condattr_init(attr: *mut pthread_condattr_t) -> c_int {
    unsafe { attr.cast::<RlctCondAttr>().write(RlctCondAttr::default()) };
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_condattr_setclock.html>.
///
/// Sets the clock attribute in the attributes object referenced by `attr`.
/// Upon success, returns `0`. Upon failure, an error number is returned to
/// indicated the error.
///
/// # Implementation
/// Always successful, so will never return an error number.
///
/// # Safety
/// It is undefined behaviour if `attr` is uninitialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_condattr_setclock(
    attr: *mut pthread_condattr_t,
    clock_id: clockid_t,
) -> c_int {
    // TODO return EINVAL if clock_id is invalid or a CPU-time clock
    (unsafe { *attr.cast::<RlctCondAttr>() }).clock = clock_id;
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_condattr_setpshared.html>.
///
/// Sets the process-shared attribute in the attributes object referenced by
/// `attr`.
///
/// Upon success, returns `0`. Upon failure, an error number is returned to
/// indicate the error.
///
/// # Safety
/// It is undefined behaviour if `attr` is uninitialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_condattr_setpshared(
    attr: *mut pthread_condattr_t,
    pshared: c_int,
) -> c_int {
    match pshared {
        PTHREAD_PROCESS_SHARED | PTHREAD_PROCESS_PRIVATE => {
            (unsafe { *attr.cast::<RlctCondAttr>() }).pshared = pshared;
            0
        }
        _ => crate::header::errno::EINVAL,
    }
}

pub(crate) type RlctCondAttr = crate::sync::cond::CondAttr;

pub(crate) type RlctCond = crate::sync::cond::Cond;
