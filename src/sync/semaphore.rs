// From https://www.remlab.net/op/futex-misc.shtml
//TODO: improve implementation

use crate::{
    error::{Errno, Result},
    header::{
        errno,
        time::{CLOCK_MONOTONIC, CLOCK_REALTIME, timespec, timespec_realtime_to_monotonic},
    },
    platform::{
        Pal, Sys,
        types::{c_uint, clockid_t},
    },
    sync::FutexAtomicTy,
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

    pub fn try_wait(&self) -> bool {
        loop {
            let value = self.count.load(Ordering::SeqCst);

            if value == 0 {
                return false;
            }

            if self
                .count
                .compare_exchange_weak(value, value - 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                // Acquired
                return true;
            }
            // Try again (as long as value > 0)
        }
    }

    pub fn wait(&self, timeout_opt: Option<&timespec>, clock_id: clockid_t) -> Result<()> {
        loop {
            if self.try_wait() {
                return Ok(());
            }
            // value must be zero
            if let Some(timeout) = timeout_opt {
                let relative = match clock_id {
                    // FUTEX expect monotonic clock
                    CLOCK_MONOTONIC => timeout.clone(),
                    CLOCK_REALTIME => timespec_realtime_to_monotonic(timeout)?,
                    _ => return Err(Errno(errno::EINVAL)),
                };
                unsafe { Sys::futex_wait(self.count.ptr(), 0, Some(&relative))? };
            } else {
                // Use futex to wait for the next change, without a timeout
                unsafe { Sys::futex_wait(self.count.ptr(), 0, None)? };
            }
        }
    }
    pub fn value(&self) -> c_uint {
        self.count.load(Ordering::SeqCst)
    }
}
