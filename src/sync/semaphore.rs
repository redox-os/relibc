// From https://www.remlab.net/op/futex-misc.shtml
//TODO: improve implementation

use crate::{
    header::time::{CLOCK_MONOTONIC, CLOCK_REALTIME, timespec, timespec_realtime_to_monotonic},
    platform::types::{c_uint, clockid_t},
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

            if value == 0 {
                return 0;
            }

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

    pub fn wait(&self, timeout_opt: Option<&timespec>, clock_id: clockid_t) -> Result<(), ()> {
        loop {
            let value = self.try_wait();

            if value == 0 {
                return Ok(());
            }

            if let Some(timeout) = timeout_opt {
                let relative = match clock_id {
                    // FUTEX expect monotonic clock
                    CLOCK_MONOTONIC => timeout.clone(),
                    CLOCK_REALTIME => match timespec_realtime_to_monotonic(timeout.clone()) {
                        Ok(relative) => relative,
                        Err(_) => return Err(()),
                    },
                    _ => return Err(()),
                };
                crate::sync::futex_wait(&self.count, value, Some(&relative));
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
