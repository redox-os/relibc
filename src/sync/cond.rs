// Used design from https://www.remlab.net/op/futex-condvar.shtml

use crate::{
    error::Errno,
    header::{
        errno::{EINVAL, ENOMEM, ETIMEDOUT},
        pthread::*,
        time::{CLOCK_MONOTONIC, CLOCK_REALTIME, clock_gettime, timespec},
    },
    platform::types::clockid_t,
};

use core::sync::atomic::{AtomicU32 as AtomicUint, Ordering};

#[derive(Clone, Copy)]
pub struct CondAttr {
    pub clock: clockid_t,
    pub pshared: i32,
}

impl Default for CondAttr {
    fn default() -> Self {
        Self {
            // defaults according to POSIX
            clock: CLOCK_REALTIME,            // for timedwait
            pshared: PTHREAD_PROCESS_PRIVATE, // TODO
        }
    }
}

pub struct Cond {
    cur: AtomicUint,
    prev: AtomicUint,
}

type Result<T, E = Errno> = core::result::Result<T, E>;

impl Default for Cond {
    fn default() -> Self {
        Self::new()
    }
}

impl Cond {
    pub fn new() -> Self {
        Self {
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
        self.broadcast()
        //self.wake(1)
    }
    pub fn clockwait(
        &self,
        mutex: &RlctMutex,
        timeout: &timespec,
        clock_id: clockid_t,
    ) -> Result<(), Errno> {
        // adjusted timeout similar in semaphore.rs

        let relative = match clock_id {
            // FUTEX expect monotonic clock
            CLOCK_MONOTONIC => timeout.clone(),
            CLOCK_REALTIME => {
                let mut realtime = timespec::default();
                unsafe { clock_gettime(CLOCK_REALTIME, &mut realtime) };
                let mut monotonic = timespec::default();
                unsafe { clock_gettime(CLOCK_MONOTONIC, &mut monotonic) };
                let Some(delta) = timespec::subtract(timeout.clone(), realtime) else {
                    return Err(Errno(ETIMEDOUT));
                };
                let Some(relative) = timespec::add(monotonic, delta) else {
                    return Err(Errno(ENOMEM));
                };
                relative
            }
            _ => return Err(Errno(EINVAL)),
        };

        self.wait_inner(mutex, Some(&relative))
    }
    pub fn timedwait(&self, mutex: &RlctMutex, timeout: &timespec) -> Result<(), Errno> {
        // TODO: The clock can be other than CLOCK_REALTIME depends on CondAttr
        self.clockwait(mutex, timeout, CLOCK_REALTIME)
    }
    fn wait_inner(&self, mutex: &RlctMutex, timeout: Option<&timespec>) -> Result<(), Errno> {
        self.wait_inner_generic(|| mutex.unlock(), || mutex.lock(), timeout)
    }
    pub fn wait_inner_typedmutex<'lock, T>(
        &self,
        guard: crate::sync::MutexGuard<'lock, T>,
    ) -> crate::sync::MutexGuard<'lock, T> {
        let mut newguard = None;
        let lock = guard.mutex;
        self.wait_inner_generic(
            move || {
                drop(guard);
                Ok(())
            },
            || {
                newguard = Some(lock.lock());
                Ok(())
            },
            None,
        )
        .unwrap();
        newguard.unwrap()
    }
    // TODO: FUTEX_REQUEUE
    fn wait_inner_generic(
        &self,
        unlock: impl FnOnce() -> Result<()>,
        lock: impl FnOnce() -> Result<()>,
        deadline: Option<&timespec>,
    ) -> Result<(), Errno> {
        // TODO: Error checking for certain types (i.e. robust and errorcheck) of mutexes, e.g. if the
        // mutex is not locked.
        let current = self.cur.load(Ordering::Relaxed);
        self.prev.store(current, Ordering::Relaxed);

        unlock()?;
        let futex_r = crate::sync::futex_wait(&self.cur, current, deadline);
        lock()?;

        match futex_r {
            super::FutexWaitResult::Waited => Ok(()),
            super::FutexWaitResult::Stale => Ok(()),
            super::FutexWaitResult::TimedOut => Err(Errno(ETIMEDOUT)),
        }
    }
    pub fn wait(&self, mutex: &RlctMutex) -> Result<(), Errno> {
        self.wait_inner(mutex, None)
    }
}
