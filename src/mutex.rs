use core::cell::UnsafeCell;
use core::intrinsics;
use core::ops::{Deref, DerefMut};
use core::sync::atomic;
use platform::{Pal, Sys};
use platform::types::*;

pub const FUTEX_WAIT: c_int = 0;
pub const FUTEX_WAKE: c_int = 1;

pub struct Mutex<T> {
    lock: UnsafeCell<c_int>,
    content: UnsafeCell<T>
}
unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}
impl<T> Mutex<T> {
    /// Create a new mutex
    pub fn new(content: T) -> Self {
        Self {
            lock: UnsafeCell::new(0),
            content: UnsafeCell::new(content)
        }
    }

    /// Tries to lock the mutex, fails if it's already locked. Manual means
    /// it's up to you to unlock it after mutex. Returns the last atomic value
    /// on failure. You should probably not worry about this, it's used for
    /// internal optimizations.
    pub unsafe fn manual_try_lock(&self) -> Result<&mut T, c_int> {
        let value = intrinsics::atomic_cxchg(self.lock.get(), 0, 1).0;
        if value == 0 {
            return Ok(&mut *self.content.get());
        }
        Err(value)
    }
    /// Lock the mutex, returning the inner content. After doing this, it's
    /// your responsibility to unlock it after usage. Mostly useful for FFI:
    /// Prefer normal .lock() where possible.
    pub unsafe fn manual_lock(&self) -> &mut T {
        let mut last = 0;

        // First, try spinning for really short durations:
        for _ in 0..100 {
            atomic::spin_loop_hint();
            last = match self.manual_try_lock() {
                Ok(content) => return content,
                Err(value) => value
            };
        }

        // We're waiting for a longer duration, so let's employ a futex.
        loop {
            // If the value is 1, set it to 2 to signify that we're waiting for
            // it to to send a FUTEX_WAKE on unlock.
            //
            // - Skip the atomic operation if the last value was 2, since it most likely hasn't changed.
            // - Skip the futex wait if the atomic operation says the mutex is unlocked.
            if last == 2 || intrinsics::atomic_cxchg(self.lock.get(), 1, 2).0 != 0 {
                Sys::futex(self.lock.get(), FUTEX_WAIT, 2);
            }

            last = match self.manual_try_lock() {
                Ok(content) => return content,
                Err(value) => value
            };
        }
    }
    /// Unlock the mutex, if it's locked.
    pub unsafe fn manual_unlock(&self) {
        if intrinsics::atomic_xchg(self.lock.get(), 0) == 2 {
            // At least one futex is up, so let's notify it
            Sys::futex(self.lock.get(), FUTEX_WAKE, 1);
        }
    }

    /// Tries to lock the mutex and returns a guard that automatically unlocks
    /// the mutex when it falls out of scope.
    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        unsafe {
            self.manual_try_lock().ok().map(|content| MutexGuard {
                mutex: self,
                content
            })
        }
    }
    /// Locks the mutex and returns a guard that automatically unlocks the
    /// mutex when it falls out of scope.
    pub fn lock(&self) -> MutexGuard<T> {
        MutexGuard {
            mutex: self,
            content: unsafe { self.manual_lock() }
        }
    }
}

pub struct MutexGuard<'a, T: 'a> {
    mutex: &'a Mutex<T>,
    content: &'a mut T
}
impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}
impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.content
    }
}
impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        unsafe {
            self.mutex.manual_unlock();
        }
    }
}
