use core::sync::atomic::{AtomicI32 as AtomicInt, Ordering};

use crate::header::errno::EBUSY;

use super::*;

const UNLOCKED: c_int = 0;
const LOCKED: c_int = 1;

#[no_mangle]
pub unsafe extern "C" fn pthread_spin_destroy(spinlock: *mut pthread_spinlock_t) -> c_int {
    let _spinlock = &mut *spinlock.cast::<RlctSpinlock>();

    // No-op
    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_spin_init(
    spinlock: *mut pthread_spinlock_t,
    _pshared: c_int,
) -> c_int {
    // TODO: pshared doesn't matter in most situations, as memory is just memory, but this may be
    // different on some architectures...

    spinlock.cast::<RlctSpinlock>().write(RlctSpinlock {
        inner: AtomicInt::new(UNLOCKED),
    });

    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_spin_lock(spinlock: *mut pthread_spinlock_t) -> c_int {
    let spinlock = &*spinlock.cast::<RlctSpinlock>();

    loop {
        match spinlock.inner.compare_exchange_weak(
            UNLOCKED,
            LOCKED,
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            Ok(_) => break,
            Err(_) => core::hint::spin_loop(),
        }
    }

    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_spin_trylock(spinlock: *mut pthread_spinlock_t) -> c_int {
    let spinlock = &*spinlock.cast::<RlctSpinlock>();

    match spinlock
        .inner
        .compare_exchange(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed)
    {
        Ok(_) => (),
        Err(_) => return EBUSY,
    }

    0
}
#[no_mangle]
pub unsafe extern "C" fn pthread_spin_unlock(spinlock: *mut pthread_spinlock_t) -> c_int {
    let spinlock = &*spinlock.cast::<RlctSpinlock>();

    spinlock.inner.store(UNLOCKED, Ordering::Release);

    0
}
pub(crate) struct RlctSpinlock {
    pub inner: AtomicInt,
}
