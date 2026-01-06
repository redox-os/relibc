// Used design from https://www.remlab.net/op/futex-condvar.shtml

use crate::{
    error::Errno,
    header::{
        errno::ETIMEDOUT,
        pthread::*,
        time::{CLOCK_REALTIME, clock_gettime, timespec},
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
    // based on CondAttr
    pub clock: clockid_t,
    pub pshared: i32,
}

type Result<T, E = Errno> = core::result::Result<T, E>;

impl Default for Cond {
    fn default() -> Self {
        Self::new(&CondAttr::default())
    }
}

impl Cond {
    pub fn new(attr: &CondAttr) -> Self {
        Self {
            cur: AtomicUint::new(0),
            prev: AtomicUint::new(0),
            clock: attr.clock,
            pshared: attr.pshared,
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

        let mut time = timespec::default();
        unsafe { clock_gettime(clock_id, &mut time) };
        if (time.tv_sec > timeout.tv_sec)
            || (time.tv_sec == timeout.tv_sec && time.tv_nsec >= timeout.tv_nsec)
        {
            //Timeout happened, return directly
            return Err(Errno(ETIMEDOUT));
        } else {
            let mut relative = timespec {
                tv_sec: timeout.tv_sec,
                tv_nsec: timeout.tv_nsec,
            };
            while relative.tv_nsec < time.tv_nsec {
                relative.tv_sec -= 1;
                relative.tv_nsec += 1_000_000_000;
            }
            relative.tv_sec -= time.tv_sec;
            relative.tv_nsec -= time.tv_nsec;

            self.wait_inner(mutex, Some(&relative))
        }
    }
    pub fn timedwait(&self, mutex: &RlctMutex, timeout: &timespec) -> Result<(), Errno> {
        self.clockwait(mutex, timeout, self.clock)
    }
    fn wait_inner(&self, mutex: &RlctMutex, timeout: Option<&timespec>) -> Result<(), Errno> {
        self.wait_inner_generic(
            || mutex.unlock(),
            || mutex.lock(),
            |timeout| mutex.lock_with_timeout(timeout),
            timeout,
        )
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
            |_| unreachable!(),
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
        lock_with_timeout: impl FnOnce(&timespec) -> Result<()>,
        deadline: Option<&timespec>,
    ) -> Result<(), Errno> {
        // TODO: Error checking for certain types (i.e. robust and errorcheck) of mutexes, e.g. if the
        // mutex is not locked.
        let current = self.cur.load(Ordering::Relaxed);
        self.prev.store(current, Ordering::Relaxed);

        unlock();

        match deadline {
            Some(deadline) => {
                crate::sync::futex_wait(&self.cur, current, Some(&deadline));
                lock_with_timeout(deadline);
            }
            None => {
                crate::sync::futex_wait(&self.cur, current, None);
                lock();
            }
        }
        Ok(())
    }
    pub fn wait(&self, mutex: &RlctMutex) -> Result<(), Errno> {
        self.wait_inner(mutex, None)
    }
}
