use core::cmp;
use core::num::NonZeroU32;
use core::sync::atomic::{AtomicU32 as AtomicUint, Ordering};

pub struct Barrier {
    waited_count: AtomicUint,
    notified_count: AtomicUint,
    cycles_count: AtomicUint,
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
            cycles_count: AtomicUint::new(0),
            original_count: count,
        }
    }
    pub fn wait(&self) -> WaitResult {
        // The barrier wait operation can be divided into two parts: (1) incrementing the wait count where
        // N-1 waiters wait and one notifies the rest, and (2) notifying all threads that have been
        // waiting.
        let original_count = self.original_count.get();
        let mut new = self.waited_count.fetch_add(1, Ordering::Acquire) + 1;
        let original_cycle_count = self.cycles_count.load(Ordering::Acquire);

        loop {
            let result = match Ord::cmp(&new, &original_count) {
                cmp::Ordering::Less => {
                    // new < original_count, i.e. we were one of the threads that incremented the
                    // counter, and will return without SERIAL_THREAD later, but need to continue
                    // waiting for the last waiter to notify the others.

                    loop {
                        let count = self.waited_count.load(Ordering::Acquire);

                        if count >= original_count { break }

                        let _ = crate::sync::futex_wait(&self.waited_count, count, None);
                    }

                    WaitResult::Waited
                }
                cmp::Ordering::Equal => {
                    // new == original_count, i.e. we were the one thread doing the last increment, and we
                    // will be responsible for waking up all other waiters.

                    crate::sync::futex_wake(&self.waited_count, original_count as i32 - 1);

                    WaitResult::NotifiedAll
                }
                cmp::Ordering::Greater => {
                    let mut next_cycle_count;

                    loop {
                        next_cycle_count = self.cycles_count.load(Ordering::Acquire);

                        if next_cycle_count != original_cycle_count {
                            break;
                        }

                        crate::sync::futex_wait(&self.cycles_count, next_cycle_count, None);
                    }
                    let difference = next_cycle_count.wrapping_sub(original_cycle_count);

                    new = new.saturating_sub(difference * original_cycle_count);
                    continue;
                }
            };

            if self.notified_count.fetch_add(1, Ordering::AcqRel) + 1 == original_count {
                self.notified_count.store(0, Ordering::Relaxed);
                // Cycle count can be incremented nonatomically here, as this branch can only be
                // reached once until waited_count is decremented again.
                self.cycles_count.store(self.cycles_count.load(Ordering::Acquire).wrapping_add(1), Ordering::Release);

                let _ = self.waited_count.fetch_sub(original_count, Ordering::Relaxed);

                let _ = crate::sync::futex_wake(&self.cycles_count, i32::MAX);
            }
            return result;
        }
    }
}
