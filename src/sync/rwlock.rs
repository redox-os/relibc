use core::{
    cell::UnsafeCell,
    fmt, ops,
    ptr::NonNull,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::{header::time::timespec, pthread::Pshared};

pub struct InnerRwLock {
    state: AtomicU32,
}
// PTHREAD_RWLOCK_INITIALIZER is defined as "all zeroes".

const WAITING_WR: u32 = 1 << (u32::BITS - 1);
const COUNT_MASK: u32 = WAITING_WR - 1;
const EXCLUSIVE: u32 = COUNT_MASK;

// TODO: Optimize for short waits and long waits, using AtomicLock::wait_until, but still
// supporting timeouts.
// TODO: Add futex ops that use bitmasks.

impl InnerRwLock {
    pub const fn new(_pshared: Pshared) -> Self {
        Self {
            state: AtomicU32::new(0),
        }
    }
    pub fn acquire_write_lock(&self, deadline: Option<&timespec>) {
        let mut waiting_wr = self.state.load(Ordering::Relaxed) & WAITING_WR;

        loop {
            match self.state.compare_exchange_weak(
                waiting_wr,
                EXCLUSIVE,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => return,
                Err(actual) => {
                    let expected = actual;
                    let expected = if actual & COUNT_MASK != EXCLUSIVE {
                        // Set the exclusive bit, but only if we're waiting for readers, to avoid
                        // reader starvation by overprioritizing write locks.
                        self.state.fetch_or(WAITING_WR, Ordering::Relaxed);

                        actual | WAITING_WR
                    } else {
                        actual
                    };
                    waiting_wr = expected & WAITING_WR;

                    if actual & COUNT_MASK > 0 {
                        let _ = crate::sync::futex_wait(&self.state, expected, deadline);
                    } else {
                        // We must avoid blocking indefinitely in our `futex_wait()`, in this case
                        // where it's possible that `self.state == expected` but our futex might
                        // never be woken again, because it's possible that all other threads
                        // already did their `futex_wake()` before we would've done our
                        // `futex_wait()`.
                    }
                }
            }
        }
    }
    pub fn acquire_read_lock(&self, deadline: Option<&timespec>) {
        // TODO: timeout
        while let Err(old) = self.try_acquire_read_lock() {
            crate::sync::futex_wait(&self.state, old, deadline);
        }
    }
    pub fn try_acquire_read_lock(&self) -> Result<(), u32> {
        let mut cached = self.state.load(Ordering::Acquire);

        loop {
            let waiting_wr = cached & WAITING_WR;
            let old = if cached & COUNT_MASK == EXCLUSIVE {
                0
            } else {
                cached & COUNT_MASK
            };
            let new = old + 1;

            // TODO: Return with error code instead?
            assert_ne!(
                new & COUNT_MASK,
                EXCLUSIVE,
                "maximum number of rwlock readers reached"
            );

            match self.state.compare_exchange_weak(
                (old & COUNT_MASK) | waiting_wr,
                new | waiting_wr,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(()),

                Err(value) if value & COUNT_MASK == EXCLUSIVE => return Err(value),
                Err(value) => {
                    cached = value;
                    // TODO: SCHED_YIELD?
                    core::hint::spin_loop();
                }
            }
        }
    }
    pub fn try_acquire_write_lock(&self) -> Result<(), u32> {
        let mut waiting_wr = self.state.load(Ordering::Relaxed) & WAITING_WR;

        loop {
            match self.state.compare_exchange_weak(
                waiting_wr,
                EXCLUSIVE,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(()),
                Err(actual) if actual & COUNT_MASK > 0 => return Err(actual),
                Err(can_retry) => {
                    waiting_wr = can_retry & WAITING_WR;

                    core::hint::spin_loop();
                    continue;
                }
            }
        }
    }

    pub fn unlock(&self) {
        let state = self.state.load(Ordering::Relaxed);

        if state & COUNT_MASK == EXCLUSIVE {
            // Unlocking a write lock.

            // This discards the writer-waiting bit, in order to ensure some level of fairness
            // between read and write locks.
            self.state.store(0, Ordering::Release);

            let _ = crate::sync::futex_wake(&self.state, i32::MAX);
        } else {
            // Unlocking a read lock. Subtract one from the reader count, but preserve the
            // WAITING_WR bit.

            if self.state.fetch_sub(1, Ordering::Release) & COUNT_MASK == 1 {
                let _ = crate::sync::futex_wake(&self.state, i32::MAX);
            }
        }
    }
}

pub struct RwLock<T: ?Sized> {
    inner: InnerRwLock,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for RwLock<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for RwLock<T> {}

impl<T> RwLock<T> {
    pub const fn new(val: T) -> Self {
        Self {
            inner: InnerRwLock::new(Pshared::Private),
            data: UnsafeCell::new(val),
        }
    }
}

impl<T: ?Sized> RwLock<T> {
    pub fn read(&self) -> ReadGuard<'_, T> {
        self.inner.acquire_read_lock(None);
        unsafe { ReadGuard::new(self) }
    }

    pub fn write(&self) -> WriteGuard<'_, T> {
        self.inner.acquire_write_lock(None);
        unsafe { WriteGuard::new(self) }
    }
}

pub struct ReadGuard<'a, T: ?Sized + 'a> {
    lock: &'a RwLock<T>,
}

impl<T: ?Sized> !Send for ReadGuard<'_, T> {}
unsafe impl<T: ?Sized + Sync> Sync for ReadGuard<'_, T> {}

impl<'a, T: ?Sized> ReadGuard<'a, T> {
    unsafe fn new(lock: &'a RwLock<T>) -> Self {
        Self { lock }
    }
}

impl<'a, T: ?Sized> ops::Deref for ReadGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: We have shared reference to the data.
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T: ?Sized> Drop for ReadGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.inner.unlock();
    }
}

impl<'a, T: ?Sized + fmt::Debug> fmt::Debug for ReadGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, T: ?Sized + fmt::Display> fmt::Display for ReadGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

pub struct WriteGuard<'a, T: ?Sized + 'a> {
    lock: &'a RwLock<T>,
}

impl<T: ?Sized> !Send for WriteGuard<'_, T> {}
unsafe impl<T: ?Sized + Sync> Sync for WriteGuard<'_, T> {}

impl<'a, T: ?Sized> WriteGuard<'a, T> {
    unsafe fn new(lock: &'a RwLock<T>) -> Self {
        Self { lock }
    }
}

impl<'a, T: ?Sized> ops::Deref for WriteGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: We have exclusive reference to the data.
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T: ?Sized> ops::DerefMut for WriteGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: We have exclusive reference to the data.
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T: ?Sized> Drop for WriteGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.inner.unlock();
    }
}

impl<'a, T: ?Sized + fmt::Debug> fmt::Debug for WriteGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, T: ?Sized + fmt::Display> fmt::Display for WriteGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}
