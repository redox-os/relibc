use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::AtomicI32 as AtomicInt;
use core::sync::atomic::Ordering::SeqCst;
use core::sync::atomic;
use platform::types::*;
use platform::{Pal, Sys};

pub const FUTEX_WAIT: c_int = 0;
pub const FUTEX_WAKE: c_int = 1;

pub struct Mutex<T> {
    lock: UnsafeCell<AtomicInt>,
    content: UnsafeCell<T>,
}
unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}
impl<T> Mutex<T> {
    /// Create a new mutex
    pub const fn new(content: T) -> Self {
        Self {
            lock: UnsafeCell::new(AtomicInt::new(0)),
            content: UnsafeCell::new(content),
        }
    }
    /// Create a new mutex that is already locked. This is a more
    /// efficient way to do the following:
    /// ```rust
    /// let mut mutex = Mutex::new(());
    /// mutex.manual_lock();
    /// ```
    pub unsafe fn locked(content: T) -> Self {
        Self {
            lock: UnsafeCell::new(AtomicInt::new(1)),
            content: UnsafeCell::new(content),
        }
    }

    unsafe fn atomic(&self) -> &mut AtomicInt {
        &mut *self.lock.get()
    }

    /// Tries to lock the mutex, fails if it's already locked. Manual means
    /// it's up to you to unlock it after mutex. Returns the last atomic value
    /// on failure. You should probably not worry about this, it's used for
    /// internal optimizations.
    pub unsafe fn manual_try_lock(&self) -> Result<&mut T, c_int> {
        self.atomic().compare_exchange(0, 1, SeqCst, SeqCst)
            .map(|_| &mut *self.content.get())
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
                Err(value) => value,
            };
        }

        // We're waiting for a longer duration, so let's employ a futex.
        loop {
            // If the value is 1, set it to 2 to signify that we're waiting for
            // it to to send a FUTEX_WAKE on unlock.
            //
            // - Skip the atomic operation if the last value was 2, since it most likely hasn't changed.
            // - Skip the futex wait if the atomic operation says the mutex is unlocked.
            if last == 2 || self.atomic().compare_exchange(1, 2, SeqCst, SeqCst).unwrap_or_else(|err| err) != 0 {
                Sys::futex(self.atomic().get_mut(), FUTEX_WAIT, 2);
            }

            last = match self.manual_try_lock() {
                Ok(content) => return content,
                Err(value) => value,
            };
        }
    }
    /// Unlock the mutex, if it's locked.
    pub unsafe fn manual_unlock(&self) {
        if self.atomic().swap(0, SeqCst) == 2 {
            // At least one futex is up, so let's notify it
            Sys::futex(self.atomic().get_mut(), FUTEX_WAKE, 1);
        }
    }

    /// Tries to lock the mutex and returns a guard that automatically unlocks
    /// the mutex when it falls out of scope.
    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        unsafe {
            self.manual_try_lock().ok().map(|content| MutexGuard {
                mutex: self,
                content,
            })
        }
    }
    /// Locks the mutex and returns a guard that automatically unlocks the
    /// mutex when it falls out of scope.
    pub fn lock(&self) -> MutexGuard<T> {
        MutexGuard {
            mutex: self,
            content: unsafe { self.manual_lock() },
        }
    }
}

pub struct MutexGuard<'a, T: 'a> {
    mutex: &'a Mutex<T>,
    content: &'a mut T,
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
