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
    pub fn timedwait(&self, mutex: &RlctMutex, timeout: &timespec) -> Result<(), Errno> {
        self.wait_inner(mutex, Some(timeout))
    }
    fn wait_inner(&self, mutex: &RlctMutex, timeout: Option<&timespec>) -> Result<(), Errno> {
        // TODO: Error checking for certain types (i.e. robust and errorcheck) of mutexes, e.g. if the
        // mutex is not locked.
        let current = self.cur.load(Ordering::Relaxed);
        self.prev.store(current, Ordering::Relaxed); // TODO: ordering?

        mutex.unlock();

        match timeout {
            Some(timeout) => {
                crate::sync::futex_wait(&self.cur, current, timespec::subtract(*timeout, crate::sync::rttime()).as_ref());
                mutex.lock_with_timeout(timeout);
            }
            None => {
                crate::sync::futex_wait(&self.cur, current, None);
                mutex.lock();
            }
        }
        Ok(())
    }
    pub fn wait(&self, mutex: &RlctMutex) -> Result<(), Errno> {
        self.wait_inner(mutex, None)
    }
}
