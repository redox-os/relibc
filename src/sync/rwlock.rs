use core::sync::atomic::{AtomicU32, Ordering};

use crate::{header::time::timespec, pthread::Pshared};

pub struct Rwlock {
    state: AtomicU32,
}
// PTHREAD_RWLOCK_INITIALIZER is defined as "all zeroes".

const EXCLUSIVE: u32 = (1 << (u32::BITS - 1)) - 1;
// Separate "waiting for wrlocks" and "waiting for rdlocks"?
//const WAITING: u32 = 1 << (u32::BITS - 1);

// TODO: Optimize for short waits and long waits, using AtomicLock::wait_until, but still
// supporting timeouts.
// TODO: Add futex ops that use bitmasks.

impl Rwlock {
    pub fn new(_pshared: Pshared) -> Self {
        Self {
            state: AtomicU32::new(0),
        }
    }
    pub fn acquire_write_lock(&self, _timeout: Option<&timespec>) {
        // TODO: timeout
        while let Err(old) = self.try_acquire_read_lock() {
            crate::sync::futex_wait(&self.state, old, None);
        }
    }
    pub fn acquire_read_lock(&self, _timeout: Option<&timespec>) {
        // TODO: timeout
        while let Err(old) = self.try_acquire_write_lock() {
            crate::sync::futex_wait(&self.state, old, None);
        }
    }
    pub fn try_acquire_read_lock(&self) -> Result<(), u32> {
        let mut cached = self.state.load(Ordering::Acquire);

        loop {
            let old = if cached == EXCLUSIVE { 0 } else { cached };
            let new = old + 1;

            // TODO: Return with error code instead?
            assert_ne!(new, EXCLUSIVE, "maximum number of rwlock readers reached");

            match self.state.compare_exchange_weak(
                cached,
                new,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(()),

                Err(value) if value == EXCLUSIVE => return Err(EXCLUSIVE),
                Err(value) => {
                    cached = value;
                    // TODO: SCHED_YIELD?
                    core::hint::spin_loop();
                }
            }
        }
    }
    pub fn try_acquire_write_lock(&self) -> Result<(), u32> {
        self.state
            .compare_exchange(0, EXCLUSIVE, Ordering::Acquire, Ordering::Relaxed)
            .map(|_| ())
    }

    pub fn unlock(&self) {
        let old = self.state.swap(0, Ordering::Release);

        if old == EXCLUSIVE {
            crate::sync::futex_wake(&self.state, i32::MAX);
        }
    }
}
