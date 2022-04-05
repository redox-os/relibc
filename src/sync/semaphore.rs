// From https://www.remlab.net/op/futex-misc.shtml
//TODO: improve implementation

use super::AtomicLock;
use crate::header::time::timespec;
use crate::platform::{types::*, Pal, Sys};
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

    pub fn post(&self) {
        self.lock.fetch_add(1, Ordering::Relaxed);
        self.lock.notify_one();
    }

    pub fn wait(&self, timeout_opt: Option<&timespec>) {
        let mut value = 1;

        loop {
            match self.lock.compare_exchange_weak(
                value,
                value - 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(ok) => return,
                Err(err) => {
                    value = err;
                }
            }

            if value == 0 {
                self.lock.wait_if(0, timeout_opt);
                value = 1;
            }
        }
    }
}
