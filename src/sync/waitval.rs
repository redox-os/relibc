use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    sync::atomic::{AtomicU32 as AtomicUint, Ordering},
};

use super::*;

/// An unsafe "one thread to one thread" synchronization primitive. Used for and modeled after
/// pthread_join only, at the moment.
#[derive(Debug)]
pub struct Waitval<T> {
    state: AtomicUint,
    value: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Send + Sync> Send for Waitval<T> {}
unsafe impl<T: Send + Sync> Sync for Waitval<T> {}

impl<T> Waitval<T> {
    pub const fn new() -> Self {
        Self {
            state: AtomicUint::new(0),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    // SAFETY: Caller must ensure both (1) that the value has not yet been initialized, and (2)
    // that this is never run by more than one thread simultaneously.
    pub unsafe fn post(&self, value: T) {
        unsafe { self.value.get().write(MaybeUninit::new(value)) };
        self.state.store(1, Ordering::Release);
        crate::sync::futex_wake(&self.state, i32::MAX);
    }

    pub fn wait(&self) -> &T {
        while self.state.load(Ordering::Acquire) == 0 {
            crate::sync::futex_wait(&self.state, 0, None);
        }

        unsafe { (*self.value.get()).assume_init_ref() }
    }
}
