use super::{AtomicLock, AttemptStatus};
use crate::platform::types::*;
use core::{cell::UnsafeCell, mem::MaybeUninit};
use core::sync::atomic::{AtomicI32 as AtomicInt, Ordering};

const UNINITIALIZED: c_int = 0;
const INITIALIZING: c_int = 1;
const WAITING: c_int = 2;
const INITIALIZED: c_int = 3;

pub struct Once<T> {
    status: AtomicInt,
    data: UnsafeCell<MaybeUninit<T>>,
}

// SAFETY:
//
// Sending a Once is the same as sending a (wrapped) T.
unsafe impl<T: Send> Send for Once<T> {}

// SAFETY:
//
// For Once to be shared between threads without being unsound, only call_once needs to be safe, at
// the moment.
//
// Send requirement: the thread that gets to run the initializer function, will put a T in the cell
// which can then be accessed by other threads, thus T needs to be send.
//
// Sync requirement: after call_once has been called, it returns the value via &T, which naturally
// forces T to be Sync.
unsafe impl<T: Send + Sync> Sync for Once<T> {}

impl<T> Once<T> {
    pub const fn new() -> Self {
        Self {
            status: AtomicInt::new(UNINITIALIZED),
            data: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
    pub fn call_once(&self, constructor: impl FnOnce() -> T) -> &T {
        match self.status.compare_exchange(
            UNINITIALIZED,
            INITIALIZING,
            // SAFETY: Success ordering: if the CAS succeeds, we technically need no
            // synchronization besides the Release store to INITIALIZED, and Acquire here forbids
            // possible loads in f() to be re-ordered before this CAS. One could argue whether or
            // not that is reasonable, but the main point is that the success ordering must be at
            // least as strong as the failure ordering.
            Ordering::Acquire,
            // SAFETY: Failure ordering: if the CAS fails, and status was INITIALIZING | WAITING,
            // then Relaxed is sufficient, as it will have to be Acquire-loaded again later. If
            // INITIALIZED is encountered however, it will nonatomically read the value in the
            // Cell, which necessitates Acquire.
            Ordering::Acquire
            // TODO: On archs where this matters, use Relaxed and core::sync::atomic::fence?
        ) {
            Ok(_must_be_uninit) => {
                // We now have exclusive access to the cell, let's initiate things!
                unsafe { self.data.get().cast::<T>().write(constructor()) };

                // Mark the data as initialized
                if self.status.swap(INITIALIZED, Ordering::Release) == WAITING {
                    // At least one thread is waiting on this to finish
                    crate::sync::futex_wake(&self.status, i32::MAX);
                }
            }
            Err(INITIALIZING) | Err(WAITING) => crate::sync::wait_until_generic(
                &self.status,
                // SAFETY: An Acquire load is necessary for the nonatomic store by the thread
                // running the constructor, to become visible.
                |status| match status.load(Ordering::Acquire) {
                    WAITING => AttemptStatus::Waiting,
                    INITIALIZED => AttemptStatus::Desired,
                    _ => AttemptStatus::Other,
                },
                // SAFETY: Double-Acquire is necessary here as well, because if the CAS fails and
                // it was INITIALIZED, the nonatomic write by the constructor thread, must be
                // visible.
                |status| match status
                    .compare_exchange_weak(INITIALIZING, WAITING, Ordering::Acquire, Ordering::Acquire)
                    .unwrap_or_else(|e| e)
                {
                    WAITING => AttemptStatus::Waiting,
                    INITIALIZED => AttemptStatus::Desired,
                    _ => AttemptStatus::Other,
                },
                WAITING,
            ),
            Err(INITIALIZED) => (),

            // TODO: Only for debug builds?
            Err(_) => unreachable!("invalid state for Once<T>"),
        }

        // At this point the data must be initialized!
        unsafe { (&*self.data.get()).assume_init_ref() }
    }
}
impl<T> Default for Once<T> {
    fn default() -> Self {
        Self::new()
    }
}
// TODO: Drop doesn't work well in const fn, instead use a wrapper for relibc Rust code that adds
// Drop, and don't use that wrapper when writing the header file impls.
/*
impl<T> Drop for Once<T> {
    fn drop(&mut self) {
        unsafe {
            if *self.status.get_mut() == INITIALIZED {
                // SAFETY: It must be initialized, because of the above condition.
                self.data.get_mut().assume_init_drop();
            }
        }
    }
}
*/
