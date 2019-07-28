use super::{AtomicLock, AttemptStatus};
use crate::platform::types::*;
use core::{cell::UnsafeCell, mem::MaybeUninit, sync::atomic::Ordering::SeqCst};

const UNINITIALIZED: c_int = 0;
const INITIALIZING: c_int = 1;
const WAITING: c_int = 2;
const INITIALIZED: c_int = 3;

pub struct Once<T> {
    status: AtomicLock,
    data: UnsafeCell<MaybeUninit<T>>,
}
unsafe impl<T: Send> Send for Once<T> {}
unsafe impl<T: Send> Sync for Once<T> {}
impl<T> Once<T> {
    pub const fn new() -> Self {
        Self {
            status: AtomicLock::new(UNINITIALIZED),
            data: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
    pub fn call_once<F>(&self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        match self
            .status
            .compare_and_swap(UNINITIALIZED, INITIALIZING, SeqCst)
        {
            UNINITIALIZED => {
                // We now have a lock, let's initiate things!
                let ret = unsafe { &mut *self.data.get() }.write(f());

                // Mark the data as initialized
                if self.status.swap(INITIALIZED, SeqCst) == WAITING {
                    // At least one thread is waiting on this to finish
                    self.status.notify_all();
                }
            }
            INITIALIZING | WAITING => self.status.wait_until(
                |lock| match lock.load(SeqCst) {
                    WAITING => AttemptStatus::Waiting,
                    INITIALIZED => AttemptStatus::Desired,
                    _ => AttemptStatus::Other,
                },
                |lock| match lock
                    .compare_exchange_weak(INITIALIZING, WAITING, SeqCst, SeqCst)
                    .unwrap_or_else(|e| e)
                {
                    WAITING => AttemptStatus::Waiting,
                    INITIALIZED => AttemptStatus::Desired,
                    _ => AttemptStatus::Other,
                },
                WAITING,
            ),
            INITIALIZED => (),
            _ => unreachable!("invalid state for Once<T>"),
        }

        // At this point the data must be initialized!
        unsafe { &mut *(&mut *self.data.get()).as_mut_ptr() }
    }
}
impl<T> Default for Once<T> {
    fn default() -> Self {
        Self::new()
    }
}
