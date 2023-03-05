// Semaphores need one post per wait, Once doesn't have separate post and wait, and while mutexes
// wait for releasing the lock, it calls notify_one and needs locking again to access the value.

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::Ordering;

use super::*;

pub struct Waitval<T> {
    state: AtomicLock,
    value: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Send + Sync> Send for Waitval<T> {}
unsafe impl<T: Send + Sync> Sync for Waitval<T> {}

impl<T> Waitval<T> {
    pub const fn new() -> Self {
        Self {
            state: AtomicLock::new(0),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    pub unsafe fn post(&self, value: T) {
        self.value.get().write(MaybeUninit::new(value));
        self.state.store(1, Ordering::Release);
        self.state.notify_all();
    }

    pub fn wait(&self) -> &T {
        while self.state.load(Ordering::Acquire) == 0 {
            self.state.wait_if(0, None);
        }

        unsafe { (*self.value.get()).assume_init_ref() }
    }
}
