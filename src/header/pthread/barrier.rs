use crate::header::errno::EINVAL;

use core::num::NonZeroU32;

use crate::sync::barrier::{Barrier, WaitResult};

use super::{
    PTHREAD_BARRIER_SERIAL_THREAD, PTHREAD_PROCESS_PRIVATE, c_int, c_uint, pthread_barrier_t,
    pthread_barrierattr_t,
};

pub(crate) type RlctBarrier = Barrier;

#[derive(Clone, Copy)]
pub(crate) struct RlctBarrierAttr {
    pshared: c_int,
}
impl Default for RlctBarrierAttr {
    fn default() -> Self {
        // pshared = PTHREAD_PROCESS_PRIVATE is default according to POSIX.
        Self {
            pshared: PTHREAD_PROCESS_PRIVATE,
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_barrier_destroy.html>.
///
/// Destroys the barrier referenced by `barrier` and releases any resources
/// used by the barrier.
///
/// Upon success, returns `0`. Upon error, returns an error number.
///
/// # Implementation
/// Not async-signal-safe.
///
/// # Safety
/// The following will result in undefined behaviour:
/// - Use of `barrier` after this function unless reinitialized by
///   `pthread_barrier_init()`.
/// - This function is called with an uninitialized `barrier`.
/// - This function is called when any thread is blocked on `barrier`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_barrier_destroy(barrier: *mut pthread_barrier_t) -> c_int {
    // No-op, currently.
    unsafe { core::ptr::drop_in_place(barrier.cast::<RlctBarrier>()) };

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_barrier_init.html>.
///
/// Allocates any resources required to use the barrier referenced by `barrier`
/// and shall initialize the barrier with attributes referenced by `attr`.
///
/// Upon success, returns `0`. Upon error, returns an error number.
///
/// # Implementation
/// Not async-signal-safe.
///
/// # Safety
/// The following will result in undefined behaviour:
/// - Use of `barrier` before this function is called.
/// - This function is called with an already initialized `barrier`.
/// - This function is called when any thread is blocked on `barrier`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_barrier_init(
    barrier: *mut pthread_barrier_t,
    attr: *const pthread_barrierattr_t,
    count: c_uint,
) -> c_int {
    let attr = unsafe { attr.cast::<RlctBarrierAttr>().as_ref() }
        .copied()
        .unwrap_or_default();

    let Some(count) = NonZeroU32::new(count) else {
        return EINVAL;
    };

    unsafe { barrier.cast::<RlctBarrier>().write(RlctBarrier::new(count)) };
    0
}

fn unlikely(condition: bool) -> bool {
    condition
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_barrier_wait.html>.
///
/// Synchronizes participating threads at the barrier referenced by `barrier`.
///
/// Upon success, returns `PTHREAD_BARRIER_SERIAL_THREAD` for a single
/// (arbitrary) thread synchronized at the barrier and `0` for each of the
/// other threads. Upon failure, returns an error number.
///
/// # Implementation
/// Not async-signal-safe.
///
/// # Safety
/// Undefined behaviour will occur if `barrier` is uninitialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_barrier_wait(barrier: *mut pthread_barrier_t) -> c_int {
    let barrier = unsafe { &*barrier.cast::<RlctBarrier>() };

    match barrier.wait() {
        WaitResult::NotifiedAll => PTHREAD_BARRIER_SERIAL_THREAD,
        WaitResult::Waited => 0,
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_barrierattr_init.html>.
///
/// Initializes a barrier attributes object `attr` with the default value of
/// all of the attributes defined by the implementation.
///
/// Upon success, returns `0`. Upon error, returns an error number.
///
/// # Implementation
/// Not async-signal-safe.
///
/// # Safety
/// Undefined behaviour will occur if `attr` is already initialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_barrierattr_init(attr: *mut pthread_barrierattr_t) -> c_int {
    unsafe { core::ptr::write(attr.cast::<RlctBarrierAttr>(), RlctBarrierAttr::default()) };

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_barrierattr_setpshared.html>.
///
/// Sets the process-shared attribute in an initialized attributes object
/// referenced by `attr`.
///
/// Upon success, returns `0`. Upon error, returns an error number.
///
/// # Implementation
/// Not async-signal-safe.
///
/// # Safety
/// Undefined behaviour will occur if `attr` is uninitialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_barrierattr_setpshared(
    attr: *mut pthread_barrierattr_t,
    pshared: c_int,
) -> c_int {
    unsafe {
        (*attr.cast::<RlctBarrierAttr>()).pshared = pshared;
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_barrierattr_getpshared.html>.
///
/// Obtains the value of the process-shared attribute from the attributes
/// object referenced by `attr`.
///
/// Upon success, returns `0` and stores the value of the process-shared
/// attribute of `attr` into the object referenced by `pshared`. Upon error,
/// returns an error number.
///
/// # Implementation
/// Not async-signal-safe.
///
/// # Safety
/// Undefined behaviour will occur if `attr` is uninitialized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_barrierattr_getpshared(
    attr: *const pthread_barrierattr_t,
    pshared: *mut c_int,
) -> c_int {
    unsafe { core::ptr::write(pshared, (*attr.cast::<RlctBarrierAttr>()).pshared) };
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_barrierattr_destroy.html>.
///
/// Destroys a barrier attributes object.
///
/// Upon success, returns `0`. Upon error, returns an error number.
///
/// # Implementation
/// Not async-signal-safe.
///
/// # Safety
/// Undefined behaviour will occur if `attr` is uninitialized or if `attr` is
/// used after calling this function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_barrierattr_destroy(attr: *mut pthread_barrierattr_t) -> c_int {
    unsafe { core::ptr::drop_in_place(attr) };
    0
}
