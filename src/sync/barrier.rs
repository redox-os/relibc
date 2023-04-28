use core::cmp;
use core::num::NonZeroU32;
use core::sync::atomic::{AtomicU32 as AtomicUint, Ordering};

pub struct Barrier {
    original_count: NonZeroU32,
    // 4
    lock: crate::sync::Mutex<Inner>,
    // 16
    cvar: crate::header::pthread::RlctCond,
    // 24
}
struct Inner {
    count: u32,
    gen_id: u32,
}

pub enum WaitResult {
    Waited,
    NotifiedAll,
}

impl Barrier {
    pub fn new(count: NonZeroU32) -> Self {
        Self {
            original_count: count,
            lock: crate::sync::Mutex::new(Inner { count: 0, gen_id: 0 }),
            cvar: crate::header::pthread::RlctCond::new(),
        }
    }
    pub fn wait(&self) -> WaitResult {
        let mut guard = self.lock.lock();
        let Inner { count, gen_id } = *guard;
        let last = self.original_count.get() - 1;

        if count == last {
            guard.gen_id = guard.gen_id.wrapping_add(1);
            guard.count = 0;

            self.cvar.broadcast();

            WaitResult::NotifiedAll
        } else {
            guard.count += 1;

            while guard.count != last && guard.gen_id == gen_id {
                guard = self.cvar.wait_inner_typedmutex(guard);
            }

            WaitResult::Waited
        }
    }
}
