use core::num::NonZeroU32;

use crate::header::pthread::PTHREAD_PROCESS_PRIVATE;

pub struct Barrier {
    original_count: NonZeroU32,
    // 4
    lock: crate::sync::Mutex<Inner>,
    // 16
    cvar: crate::sync::cond::Cond,
    // 32
}

#[derive(Clone, Copy)]
pub struct BarrierAttr {
    pub pshared: i32,
}

impl Default for BarrierAttr {
    fn default() -> Self {
        // same default with CondAttr
        Self {
            pshared: PTHREAD_PROCESS_PRIVATE,
        }
    }
}

#[derive(Debug)]
struct Inner {
    count: u32,
    // TODO: Overflows might be problematic... 64-bit?
    gen_id: u32,
}

pub enum WaitResult {
    Waited,
    NotifiedAll,
}

impl Barrier {
    pub fn new(count: NonZeroU32, attr: BarrierAttr) -> Self {
        Self {
            original_count: count,
            lock: crate::sync::Mutex::new(Inner {
                count: 0,
                gen_id: 0,
            }),
            cvar: crate::sync::cond::Cond::new(&super::cond::CondAttr {
                // assuming clock never used
                clock: 0,
                pshared: attr.pshared,
            }),
        }
    }
    pub fn wait(&self) -> WaitResult {
        let mut guard = self.lock.lock();
        let gen_id = guard.gen_id;

        guard.count += 1;

        if guard.count == self.original_count.get() {
            guard.gen_id = guard.gen_id.wrapping_add(1);
            guard.count = 0;
            self.cvar.broadcast();

            drop(guard);

            WaitResult::NotifiedAll
        } else {
            while guard.gen_id == gen_id {
                guard = self.cvar.wait_inner_typedmutex(guard);
            }

            WaitResult::Waited
        }
        /*
        let mut guard = self.lock.lock();
        let Inner { count, gen_id } = *guard;

        let last = self.original_count.get() - 1;

        if count == last {
            eprintln!("last {:?}", *guard);
            guard.gen_id = guard.gen_id.wrapping_add(1);
            guard.count = 0;

            drop(guard);

            self.cvar.broadcast();

            WaitResult::NotifiedAll
        } else {
            guard.count += 1;

            while guard.count != last && guard.gen_id == gen_id {
                eprintln!("before {:?}", *guard);
                guard = self.cvar.wait_inner_typedmutex(guard);
                eprintln!("after {:?}", *guard);
            }

            WaitResult::Waited
        }
        */
    }
}
static LOCK: crate::sync::Mutex<()> = crate::sync::Mutex::new(());
