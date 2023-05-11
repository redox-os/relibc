// From https://www.remlab.net/op/futex-misc.shtml
//TODO: improve implementation

use super::AtomicLock;
use crate::{
    header::time::{clock_gettime, timespec, CLOCK_MONOTONIC},
    platform::{types::*, Pal, Sys},
};

use core::sync::atomic::{AtomicU32, Ordering};

pub struct Semaphore {
    count: AtomicU32,
}

impl Semaphore {
    pub const fn new(value: c_uint) -> Self {
        Self {
            count: AtomicU32::new(value),
        }
    }

    // TODO: Acquire-Release ordering?

    pub fn post(&self, count: c_uint) {
        self.count.fetch_add(count, Ordering::SeqCst);
        // TODO: notify one?
        crate::sync::futex_wake(&self.count, i32::MAX);
    }

    pub fn try_wait(&self) -> u32 {
        loop {
            let value = self.count.load(Ordering::SeqCst);

            if value == 0 { return 0 }

            match self.count.compare_exchange_weak(
                value,
                value - 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => {
                    // Acquired
                    return value;
                }
                Err(_) => (),
            }
            // Try again (as long as value > 0)
        }
    }

    pub fn wait(&self, timeout_opt: Option<&timespec>) -> Result<(), ()> {
        loop {
            let value = self.try_wait();

            if value == 0 {
                return Ok(());
            }

            if let Some(timeout) = timeout_opt {
                let mut time = timespec::default();
                clock_gettime(CLOCK_MONOTONIC, &mut time);
                if (time.tv_sec > timeout.tv_sec)
                    || (time.tv_sec == timeout.tv_sec && time.tv_nsec >= timeout.tv_nsec)
                {
                    //Timeout happened, return error
                    return Err(());
                } else {
                    // Use futex to wait for the next change, with a relative timeout
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

                    crate::sync::futex_wait(&self.count, value, Some(&relative));
                }
            } else {
                // Use futex to wait for the next change, without a timeout
                crate::sync::futex_wait(&self.count, value, None);
            }
        }
    }
    pub fn value(&self) -> c_uint {
        self.count.load(Ordering::SeqCst)
    }
}
