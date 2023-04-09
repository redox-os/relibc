use crate::header::errno::*;

use core::sync::atomic::{AtomicU32 as AtomicUint, AtomicI32 as AtomicInt, Ordering};

use super::*;

pub(crate) struct RlctBarrier {
    pub count: AtomicUint,
    pub original_count: c_uint,
    pub epoch: AtomicInt,
}
pub(crate) struct RlctBarrierAttr {
    pub pshared: c_int,
}

#[no_mangle]
pub unsafe extern "C" fn pthread_barrier_destroy(barrier: *mut pthread_barrier_t) -> c_int {
    // Behavior is undefined if any thread is currently waiting.
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_barrier_init(barrier: *mut pthread_barrier_t, attr: *const pthread_barrierattr_t, count: c_uint) -> c_int {
    let attr = attr.cast::<RlctBarrierAttr>().as_ref();

    if count == 0 {
        return EINVAL;
    }

    barrier.cast::<RlctBarrier>().write(RlctBarrier {
        count: AtomicUint::new(0),
        original_count: count,
        epoch: AtomicInt::new(0),
    });
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_barrier_wait(barrier: *mut pthread_barrier_t) -> c_int {
    let barrier = &*barrier.cast::<RlctBarrier>();

    // TODO: Orderings
    let mut cached = barrier.count.load(Ordering::SeqCst);

    loop {
        let new = if cached == barrier.original_count - 1 { 0 } else { cached + 1 };

        match barrier.count.compare_exchange_weak(cached, new, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => if new == 0 {
                // We reached COUNT waits, and will thus be the thread notifying every other
                // waiter.

                todo!();

                return PTHREAD_BARRIER_SERIAL_THREAD;
            } else {
                // We increased the wait count, but this was not sufficient. We will thus have to
                // wait for the epoch to tick up.
                todo!();

                return 0;
            }
            Err(value) => {
                cached = value;
                core::hint::spin_loop();
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_barrierattr_init(attr: *mut pthread_barrierattr_t) -> c_int {
    // PTHREAD_PROCESS_PRIVATE is default according to POSIX.
    core::ptr::write(attr.cast::<RlctBarrierAttr>(), RlctBarrierAttr { pshared: PTHREAD_PROCESS_PRIVATE });

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_barrierattr_setpshared(attr: *mut pthread_barrierattr_t, pshared: c_int) -> c_int {
    (*attr.cast::<RlctBarrierAttr>()).pshared = pshared;
    0
}
