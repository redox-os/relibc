// TODO: Share code for simple futex-based mutex between relibc's Mutex<()> and this.

use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU32, Ordering},
};

pub struct Mutex<T> {
    pub lockword: AtomicU32,
    pub inner: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Represents an unlocked [Mutex].
    pub const UNLOCKED: u32 = 0;
    /// Represents a locked [Mutex].
    pub const LOCKED: u32 = 1;
    /// Represents a waiting [Mutex].
    pub const WAITING: u32 = 2;

    pub const fn new(t: T) -> Self {
        Self {
            lockword: AtomicU32::new(0),
            inner: UnsafeCell::new(t),
        }
    }
    pub fn lock(&self) -> MutexGuard<'_, T> {
        while self
            .lockword
            .compare_exchange(
                Self::UNLOCKED,
                Self::LOCKED,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_err()
        {
            core::hint::spin_loop();
        }
        MutexGuard { lock: self }
    }
}
pub struct MutexGuard<'l, T> {
    lock: &'l Mutex<T>,
}
impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.inner.get() }
    }
}
impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.inner.get() }
    }
}
impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.lock
            .lockword
            .store(Mutex::<T>::UNLOCKED, Ordering::Release);
    }
}
