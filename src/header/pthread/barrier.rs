use crate::header::errno::*;

use core::num::NonZeroU32;

use crate::sync::barrier::*;

use super::*;

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

// Not async-signal-safe.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_barrier_destroy(barrier: *mut pthread_barrier_t) -> c_int {
    // Behavior is undefined if any thread is currently waiting when this is called.

    // No-op, currently.
    unsafe { core::ptr::drop_in_place(barrier.cast::<RlctBarrier>()) };

    0
}

// Not async-signal-safe.
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

// Not async-signal-safe.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_barrier_wait(barrier: *mut pthread_barrier_t) -> c_int {
    let barrier = unsafe { &*barrier.cast::<RlctBarrier>() };

    match barrier.wait() {
        WaitResult::NotifiedAll => PTHREAD_BARRIER_SERIAL_THREAD,
        WaitResult::Waited => 0,
    }
}

// Not async-signal-safe.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_barrierattr_init(attr: *mut pthread_barrierattr_t) -> c_int {
    unsafe { core::ptr::write(attr.cast::<RlctBarrierAttr>(), RlctBarrierAttr::default()) };

    0
}

// Not async-signal-safe.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_barrierattr_setpshared(
    attr: *mut pthread_barrierattr_t,
    pshared: c_int,
) -> c_int {
    (unsafe { *attr.cast::<RlctBarrierAttr>() }).pshared = pshared;
    0
}

// Not async-signal-safe.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_barrierattr_getpshared(
    attr: *const pthread_barrierattr_t,
    pshared: *mut c_int,
) -> c_int {
    unsafe { core::ptr::write(pshared, (*attr.cast::<RlctBarrierAttr>()).pshared) };
    0
}

// Not async-signal-safe.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_barrierattr_destroy(attr: *mut pthread_barrierattr_t) -> c_int {
    unsafe { core::ptr::drop_in_place(attr) };
    0
}
