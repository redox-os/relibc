use super::{AtomicLock, AttemptStatus};
use crate::platform::types::*;
use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::Ordering::SeqCst,
};

const UNLOCKED: c_int = 0;
const LOCKED: c_int = 1;
const WAITING: c_int = 2;

pub struct Mutex<T> {
    lock: AtomicLock,
    content: UnsafeCell<T>,
}
unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}
impl<T> Mutex<T> {
    /// Create a new mutex
    pub const fn new(content: T) -> Self {
        Self {
            lock: AtomicLock::new(UNLOCKED),
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
            lock: AtomicLock::new(LOCKED),
            content: UnsafeCell::new(content),
        }
    }

    /// Tries to lock the mutex, fails if it's already locked. Manual means
    /// it's up to you to unlock it after mutex. Returns the last atomic value
    /// on failure. You should probably not worry about this, it's used for
    /// internal optimizations.
    pub unsafe fn manual_try_lock(&self) -> Result<&mut T, c_int> {
        self.lock
            .compare_exchange(UNLOCKED, LOCKED, SeqCst, SeqCst)
            .map(|_| &mut *self.content.get())
    }
    /// Lock the mutex, returning the inner content. After doing this, it's
    /// your responsibility to unlock it after usage. Mostly useful for FFI:
    /// Prefer normal .lock() where possible.
    pub unsafe fn manual_lock(&self) -> &mut T {
        self.lock.wait_until(
            |lock| {
                lock.compare_exchange_weak(UNLOCKED, LOCKED, SeqCst, SeqCst)
                    .map(|_| AttemptStatus::Desired)
                    .unwrap_or_else(|e| match e {
                        WAITING => AttemptStatus::Waiting,
                        _ => AttemptStatus::Other,
                    })
            },
            |lock| match lock
                .compare_exchange_weak(LOCKED, WAITING, SeqCst, SeqCst)
                .unwrap_or_else(|e| e)
            {
                UNLOCKED => AttemptStatus::Desired,
                WAITING => AttemptStatus::Waiting,
                _ => AttemptStatus::Other,
            },
            WAITING,
        );
        &mut *self.content.get()
    }
    /// Unlock the mutex, if it's locked.
    pub unsafe fn manual_unlock(&self) {
        if self.lock.swap(UNLOCKED, SeqCst) == WAITING {
            self.lock.notify_one();
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
