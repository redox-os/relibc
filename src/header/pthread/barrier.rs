use crate::header::errno::*;

use crate::sync::AtomicLock;
use core::sync::atomic::{AtomicU32, Ordering};

use super::*;

#[repr(C)]
pub struct Barrier {
    count: AtomicU32,
    original_count: u32,
    epoch: AtomicLock,
}

#[repr(C)]
pub struct BarrierAttr {
    pshared: c_int,
}

#[no_mangle]
pub unsafe extern "C" fn pthread_barrier_destroy(barrier: *mut pthread_barrier_t) -> c_int {
    // Behavior is undefined if any thread is currently waiting.
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_barrier_init(barrier: *mut pthread_barrier_t, attr: *const pthread_barrierattr_t, count: c_uint) -> c_int {
    if count == 0 {
        return EINVAL;
    }

    core::ptr::write(barrier, Barrier {
        count: AtomicU32::new(0),
        original_count: count,
        epoch: AtomicLock::new(0),
    });
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_barrier_wait(barrier: *mut pthread_barrier_t) -> c_int {
    let barrier: &pthread_barrier_t = &*barrier;

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
    core::ptr::write(attr, BarrierAttr { pshared: PTHREAD_PROCESS_PRIVATE });

    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_barrierattr_setpshared(attr: *mut pthread_barrierattr_t, pshared: c_int) -> c_int {
    (*attr).pshared = pshared;
    0
}
