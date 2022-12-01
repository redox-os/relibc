// From https://www.remlab.net/op/futex-misc.shtml
//TODO: improve implementation

use super::AtomicLock;
use crate::{
    header::time::{CLOCK_MONOTONIC, clock_gettime, timespec},
    platform::{types::*, Pal, Sys},
};

use core::sync::atomic::Ordering;

pub struct Semaphore {
    lock: AtomicLock,
}

//TODO: fix to use futex again
impl Semaphore {
    pub const fn new(value: c_int) -> Self {
        Self {
            lock: AtomicLock::new(value),
        }
    }

    pub fn post(&self, count: c_int) {
        self.lock.fetch_add(count, Ordering::SeqCst);
    }

    pub fn try_wait(&self) -> Result<(), ()> {
        let mut value = self.lock.load(Ordering::SeqCst);
        if value > 0 {
            match self.lock.compare_exchange(
                value,
                value - 1,
                Ordering::SeqCst,
                Ordering::SeqCst
            ) {
                Ok(_) => Ok(()),
                Err(_) => Err(())
            }
        } else {
            Err(())
        }
    }

    pub fn wait(&self, timeout_opt: Option<&timespec>) -> Result<(), ()> {

        loop {
            match self.try_wait() {
                Ok(()) => {
                    return Ok(());
                }
                Err(()) => ()
            }
            if let Some(timeout) = timeout_opt {
                let mut time = timespec::default();
                clock_gettime(CLOCK_MONOTONIC, &mut time);
                if (time.tv_sec > timeout.tv_sec) ||
                   (time.tv_sec == timeout.tv_sec && time.tv_nsec >= timeout.tv_nsec)
                {
                    return Err(())
                }
            }
            Sys::sched_yield();
        }
    }
}
