use core::cmp;
use core::num::NonZeroU32;
use core::sync::atomic::{AtomicU32 as AtomicUint, Ordering};

pub struct Barrier {
    waited_count: AtomicUint,
    notified_count: AtomicUint,
    original_count: NonZeroU32,
}

pub enum WaitResult {
    Waited,
    NotifiedAll,
}

impl Barrier {
    pub fn new(count: NonZeroU32) -> Self {
        Self {
            waited_count: AtomicUint::new(0),
            notified_count: AtomicUint::new(0),
            original_count: count,
        }
    }
    pub fn wait(&self) -> WaitResult {
        // The barrier wait operation can be divided into two parts: (1) incrementing the wait count where
        // N-1 waiters wait and one notifies the rest, and (2) notifying all threads that have been
        // waiting.

        let original_count = self.original_count.get();

        loop {
            let new = self.waited_count.fetch_add(1, Ordering::Acquire) + 1;

            match Ord::cmp(&new, &original_count) {
                cmp::Ordering::Less => {
                    // new < original_count, i.e. we were one of the threads that incremented the counter,
                    // but need to continue waiting for the last waiter to notify the others.

                    loop {
                        let count = self.waited_count.load(Ordering::Acquire);

                        if count >= original_count { break }

                        let _ = crate::sync::futex_wait(&self.waited_count, count, None);
                    }

                    // When the required number of threads have called pthread_barrier_wait so waited_count
                    // >= original_count (should never be able to exceed that value), we can safely reset
                    // the counter to zero.

                    if self.notified_count.fetch_add(1, Ordering::Relaxed) + 1 >= original_count {
                        self.waited_count.store(0, Ordering::Relaxed);
                    }

                    return WaitResult::Waited;
                }
                cmp::Ordering::Equal => {
                    // new == original_count, i.e. we were the one thread doing the last increment, and we
                    // will be responsible for waking up all other waiters.

                    crate::sync::futex_wake(&self.waited_count, i32::MAX);

                    if self.notified_count.fetch_add(1, Ordering::Relaxed) + 1 >= original_count {
                        self.waited_count.store(0, Ordering::Relaxed);
                    }

                    return WaitResult::NotifiedAll;
                }
                // FIXME: Starvation?
                cmp::Ordering::Greater => {
                    let mut cached = new;
                    while cached >= original_count {
                        // new > original_count, i.e. we are waiting on a barrier that is already finished, but
                        // which has not yet awoken all its waiters and re-initialized the self. The
                        // simplest way to handle this is to wait for waited_count to return to zero, and
                        // start over.

                        crate::sync::futex_wait(&self.waited_count, cached, None);

                        cached = self.waited_count.load(Ordering::Acquire);
                    }
                }
            }
        }
    }
}
