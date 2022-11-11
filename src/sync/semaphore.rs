// From https://www.remlab.net/op/futex-misc.shtml
//TODO: improve implementation

use super::AtomicLock;
use crate::{
    header::time::timespec,
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

    pub fn post(&self) {
        self.lock.fetch_add(1, Ordering::Release);
    }

    pub fn wait(&self, timeout_opt: Option<&timespec>) {
        if let Some(timeout) = timeout_opt {
            println!(
                "semaphore wait tv_sec: {}, tv_nsec: {}",
                timeout.tv_sec, timeout.tv_nsec
            );
        }
        loop {
            while self.lock.load(Ordering::Acquire) < 1 {
                //spin_loop();
                Sys::sched_yield();
            }
            let tmp = self.lock.fetch_sub(1, Ordering::AcqRel);
            if tmp >= 1 {
                break;
            }
            self.lock.fetch_add(1, Ordering::Release);
        }
    }
}
