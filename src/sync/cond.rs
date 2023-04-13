use crate::header::pthread::*;
use crate::header::bits_pthread::*;
use crate::header::time::timespec;
use crate::pthread::Errno;

use core::sync::atomic::{AtomicU32 as AtomicUint, Ordering};

pub struct Cond {
    cur: AtomicUint,
    prev: AtomicUint,
}
impl Cond {
    pub fn new() -> Self {
        Self{
            cur: AtomicUint::new(0),
            prev: AtomicUint::new(0),
        }
    }
    fn wake(&self, count: i32) -> Result<(), Errno> {
        // This is formally correct as long as we don't have more than u32::MAX threads.
        let prev = self.prev.load(Ordering::Relaxed);
        self.cur.store(prev.wrapping_add(1), Ordering::Relaxed);

        crate::sync::futex_wake(&self.cur, count);
        Ok(())
    }
    pub fn broadcast(&self) -> Result<(), Errno> {
        self.wake(i32::MAX)
    }
    pub fn signal(&self) -> Result<(), Errno> {
        self.wake(1)
    }
    // TODO: Safe version using RlctMutexGuard?
    pub unsafe fn timedwait(&self, mutex_ptr: *mut pthread_mutex_t, timeout: Option<&timespec>) -> Result<(), Errno> {
        // TODO: Error checking for certain types (i.e. robust and errorcheck) of mutexes, e.g. if the
        // mutex is not locked.
        let current = self.cur.load(Ordering::Relaxed);
        self.prev.store(current, Ordering::Relaxed); // TODO: ordering?

        pthread_mutex_unlock(mutex_ptr);

        match timeout {
            Some(timeout) => {
                crate::sync::futex_wait(&self.cur, current, timespec::subtract(*timeout, crate::sync::rttime()).as_ref());
                pthread_mutex_timedlock(mutex_ptr, timespec::subtract(*timeout, crate::sync::rttime()).as_ref().map_or(core::ptr::null(), |r| r as *const timespec));
            }
            None => {
                crate::sync::futex_wait(&self.cur, current, None);
                pthread_mutex_lock(mutex_ptr);
            }
        }
        Ok(())
    }
    pub unsafe fn wait(&self, mutex_ptr: *mut pthread_mutex_t) -> Result<(), Errno> {
        self.timedwait(mutex_ptr, None)
    }
}
