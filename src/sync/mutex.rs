use super::{AtomicLock, AttemptStatus};
use crate::platform::types::*;
use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicI32 as AtomicInt, Ordering},
};

pub(crate) const UNLOCKED: c_int = 0;
pub(crate) const LOCKED: c_int = 1;
pub(crate) const WAITING: c_int = 2;

pub struct Mutex<T> {
    pub(crate) lock: AtomicLock,
    content: UnsafeCell<T>,
}
unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

pub(crate) unsafe fn manual_try_lock_generic(word: &AtomicInt) -> bool {
    word.compare_exchange(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed)
        .is_ok()
}
pub(crate) unsafe fn manual_lock_generic(word: &AtomicInt) {
    crate::sync::wait_until_generic(
        word,
        |lock| {
            lock.compare_exchange_weak(UNLOCKED, LOCKED, Ordering::Acquire, Ordering::Relaxed)
                .map(|_| AttemptStatus::Desired)
                .unwrap_or_else(|e| match e {
                    WAITING => AttemptStatus::Waiting,
                    _ => AttemptStatus::Other,
                })
        },
        |lock| match lock
            // TODO: Ordering
            .compare_exchange_weak(LOCKED, WAITING, Ordering::SeqCst, Ordering::SeqCst)
            .unwrap_or_else(|e| e)
        {
            UNLOCKED => AttemptStatus::Desired,
            WAITING => AttemptStatus::Waiting,
            _ => AttemptStatus::Other,
        },
        WAITING,
    );
}
pub(crate) unsafe fn manual_unlock_generic(word: &AtomicInt) {
    if word.swap(UNLOCKED, Ordering::Release) == WAITING {
        crate::sync::futex_wake(word, i32::MAX);
    }
}

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
        if unsafe { manual_try_lock_generic(&self.lock) } {
            Ok(unsafe { &mut *self.content.get() })
        } else {
            Err(0)
        }
    }
    /// Lock the mutex, returning the inner content. After doing this, it's
    /// your responsibility to unlock it after usage. Mostly useful for FFI:
    /// Prefer normal .lock() where possible.
    pub unsafe fn manual_lock(&self) -> &mut T {
        unsafe { manual_lock_generic(&self.lock) };
        unsafe { &mut *self.content.get() }
    }
    /// Unlock the mutex, if it's locked.
    pub unsafe fn manual_unlock(&self) {
        unsafe { manual_unlock_generic(&self.lock) }
    }
    pub fn as_ptr(&self) -> *mut T {
        self.content.get()
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
    pub(crate) mutex: &'a Mutex<T>,
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
