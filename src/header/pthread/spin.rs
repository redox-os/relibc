use core::sync::atomic::{AtomicI32 as AtomicInt, Ordering};

use crate::header::errno::EBUSY;

use super::*;

pub const UNLOCKED: c_int = 0;
pub const LOCKED: c_int = 1;

pub unsafe extern "C" fn pthread_spin_destroy(spinlock: *mut pthread_spinlock_t) -> c_int {
    0
}
pub unsafe extern "C" fn pthread_spin_init(spinlock: *mut pthread_spinlock_t, _pshared: c_int) -> c_int {
    // TODO: pshared doesn't matter in most situations, as memory is just memory, but this may be
    // different on some architectures...

    core::ptr::write(spinlock, pthread_spinlock_t { inner: AtomicInt::new(UNLOCKED) });

    0
}
pub unsafe extern "C" fn pthread_spin_lock(spinlock: *mut pthread_spinlock_t) -> c_int {
    let spinlock: &pthread_spinlock_t = &*spinlock;

    loop {
        match spinlock.inner.compare_exchange_weak(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed) {
            Ok(_) => return 0,
            Err(_) => core::hint::spin_loop(),
        }
    }
}
pub unsafe extern "C" fn pthread_spin_trylock(spinlock: *mut pthread_spinlock_t) -> c_int {
    let spinlock: &pthread_spinlock_t = &*spinlock;

    match spinlock.inner.compare_exchange(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed) {
        Ok(_) => 0,
        Err(_) => EBUSY,
    }
}
pub unsafe extern "C" fn pthread_spin_unlock(spinlock: *mut pthread_spinlock_t) -> c_int {
    let spinlock: &pthread_spinlock_t = &*spinlock;

    spinlock.inner.store(UNLOCKED, Ordering::Release);

    0
}
