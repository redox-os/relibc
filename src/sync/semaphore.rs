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

impl Semaphore {
    pub const fn new(value: c_int) -> Self {
        Self {
            lock: AtomicLock::new(value),
        }
    }

    pub fn post(&self, count: c_int) {
        self.lock.fetch_add(count, Ordering::SeqCst);
        self.lock.notify_all();
    }

    pub fn wait(&self, timeout_opt: Option<&timespec>) -> Result<(), ()> {
        loop {
            let value = self.lock.load(Ordering::SeqCst);
            if value > 0 {
                match self.lock.compare_exchange(
                    value,
                    value - 1,
                    Ordering::SeqCst,
                    Ordering::SeqCst
                ) {
                    Ok(_) => {
                        // Acquired
                        return Ok(());
                    }
                    Err(_) => ()
                }
                // Try again (as long as value > 0)
                continue;
            }
            if let Some(timeout) = timeout_opt {
                let mut time = timespec::default();
                clock_gettime(CLOCK_MONOTONIC, &mut time);
                if (time.tv_sec > timeout.tv_sec) ||
                   (time.tv_sec == timeout.tv_sec && time.tv_nsec >= timeout.tv_nsec)
                {
                    //Timeout happened, return error
                    return Err(())
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
                    self.lock.wait_if(value, Some(&relative));
                }
            } else {
                // Use futex to wait for the next change, without a timeout
                self.lock.wait_if(value, None);
            }
        }
    }
}
