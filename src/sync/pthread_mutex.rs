use core::cell::Cell;
use core::sync::atomic::{AtomicU32 as AtomicUint, Ordering};

use crate::header::pthread::*;
use crate::pthread::*;
use crate::header::time::timespec;
use crate::header::errno::*;
use crate::header::sys_wait::*;

use crate::platform::types::*;
use crate::platform::{Pal, Sys};

pub struct RlctMutex {
    // Actual locking word.
    inner: AtomicUint,
    recursive_count: AtomicUint,

    ty: Ty,
    robust: bool,
}

const STATE_UNLOCKED: u32 = 0;
const WAITING_BIT: u32 = 1 << 31;
const INDEX_MASK: u32 = !WAITING_BIT;

// TODO: Lower limit is probably better.
const RECURSIVE_COUNT_MAX_INCLUSIVE: u32 = u32::MAX;
// TODO: How many spins should we do before it becomes more time-economical to enter kernel mode
// via futexes?
const SPIN_COUNT: usize = 0;

impl RlctMutex {
    pub(crate) fn new(attr: &RlctMutexAttr) -> Result<Self, Errno> {
        let RlctMutexAttr { prioceiling, protocol, pshared: _, robust, ty } = *attr;

        Ok(Self {
            inner: AtomicUint::new(STATE_UNLOCKED),
            recursive_count: AtomicUint::new(0),
            robust: match robust {
                PTHREAD_MUTEX_STALLED => false,
                PTHREAD_MUTEX_ROBUST => true,

                _ => return Err(Errno(EINVAL)),
            },
            ty: match ty {
                PTHREAD_MUTEX_DEFAULT => Ty::Def,
                PTHREAD_MUTEX_ERRORCHECK => Ty::Errck,
                PTHREAD_MUTEX_RECURSIVE => Ty::Recursive,
                PTHREAD_MUTEX_NORMAL => Ty::Normal,

                _ => return Err(Errno(EINVAL)),
            }
        })
    }
    pub fn prioceiling(&self) -> Result<c_int, Errno> {
        println!("TODO: Implement pthread_getprioceiling");
        Ok(0)
    }
    pub fn replace_prioceiling(&self, _: c_int) -> Result<c_int, Errno> {
        println!("TODO: Implement pthread_setprioceiling");
        Ok(0)
    }
    pub fn make_consistent(&self) -> Result<(), Errno> {
        println!("TODO: Implement robust mutexes");
        Ok(())
    }
    fn lock_inner(&self, deadline: Option<&timespec>) -> Result<(), Errno> {
        let this_thread = os_tid_invalid_after_fork();

        let mut spins_left = SPIN_COUNT;

        loop {
            let result = self.inner.compare_exchange_weak(STATE_UNLOCKED, this_thread, Ordering::Acquire, Ordering::Relaxed);

            match result {
                // CAS succeeded
                Ok(_) => {
                    if self.ty == Ty::Recursive {
                        self.increment_recursive_count()?;
                    }
                    return Ok(());
                },
                // CAS failed, but the mutex was recursive and we already own the lock.
                Err(thread) if thread & INDEX_MASK == this_thread && self.ty == Ty::Recursive => {
                    self.increment_recursive_count()?;
                    return Ok(());
                }
                // CAS failed, but the mutex was error-checking and we already own the lock.
                Err(thread) if thread & INDEX_MASK == this_thread && self.ty == Ty::Errck => {
                    return Err(Errno(EAGAIN));
                }
                // CAS spuriously failed, simply retry the CAS. TODO: Use core::hint::spin_loop()?
                Err(thread) if thread & INDEX_MASK == 0 => continue,
                // CAS failed because some other thread owned the lock. We must now wait.
                Err(thread) => {
                    if spins_left > 0 {
                        spins_left -= 1;
                        core::hint::spin_loop();
                        continue;
                    }

                    spins_left = SPIN_COUNT;

                    let inner = self.inner.fetch_or(WAITING_BIT, Ordering::Relaxed);

                    if inner == STATE_UNLOCKED {
                        continue;
                    }

                    // If the mutex is not robust, simply futex_wait until unblocked.
                    crate::sync::futex_wait(&self.inner, inner | WAITING_BIT, None);
                }
            }
        }
    }
    pub fn lock(&self) -> Result<(), Errno> {
        self.lock_inner(None)
    }
    pub fn lock_with_timeout(&self, deadline: &timespec) -> Result<(), Errno> {
        self.lock_inner(Some(deadline))
    }
    fn increment_recursive_count(&self) -> Result<(), Errno> {
        // We don't have to worry about asynchronous signals here, since pthread_mutex_trylock
        // is not async-signal-safe.
        //
        // TODO: Maybe just use Cell? Send/Sync doesn't matter much anyway, and will be
        // protected by the lock itself anyway.

        let prev_recursive_count = self.recursive_count.load(Ordering::Relaxed);

        if prev_recursive_count == RECURSIVE_COUNT_MAX_INCLUSIVE {
            return Err(Errno(EAGAIN));
        }

        self.recursive_count.store(prev_recursive_count + 1, Ordering::Relaxed);

        Ok(())
    }
    pub fn try_lock(&self) -> Result<(), Errno> {
        let this_thread = os_tid_invalid_after_fork();

        // TODO: If recursive, omitting CAS may be faster if it is already owned by this thread.
        let result = self.inner.compare_exchange(STATE_UNLOCKED, this_thread, Ordering::Acquire, Ordering::Relaxed);

        if self.ty == Ty::Recursive {
            match result {
                Err(index) if index & INDEX_MASK != this_thread => return Err(Errno(EBUSY)),
                _ => (),
            }

            self.increment_recursive_count()?;

            return Ok(());
        }

        match result {
            Ok(_) => Ok(()),
            Err(index) if index & INDEX_MASK == this_thread && self.ty == Ty::Errck => Err(Errno(EDEADLK)),
            Err(_) => Err(Errno(EBUSY)),
        }
    }
    // Safe because we are not protecting any data.
    pub fn unlock(&self) -> Result<(), Errno> {
        if self.robust || matches!(self.ty, Ty::Recursive | Ty::Errck){
            if self.inner.load(Ordering::Relaxed) & INDEX_MASK != os_tid_invalid_after_fork() {
                return Err(Errno(EPERM));
            }

            // TODO: Is this fence correct?
            core::sync::atomic::fence(Ordering::Acquire);
        }

        if self.ty == Ty::Recursive {
            let next = self.recursive_count.load(Ordering::Relaxed) - 1;
            self.recursive_count.store(next, Ordering::Relaxed);

            if next > 0 { return Ok(()) }
        }

        let was_waiting = self.inner.swap(STATE_UNLOCKED, Ordering::Release) & WAITING_BIT != 0;

        if was_waiting {
            let _ = crate::sync::futex_wake(&self.inner, 1);
        }

        Ok(())
    }
}

#[repr(u8)]
#[derive(PartialEq)]
enum Ty {
    // The only difference between PTHREAD_MUTEX_NORMAL and PTHREAD_MUTEX_DEFAULT appears to be
    // that "normal" mutexes deadlock if locked multiple times on the same thread, whereas
    // "default" mutexes are UB in that case. So we can treat them as being the same type.
    Normal,
    Def,

    Errck,
    Recursive,
}

// Children after fork can only call async-signal-safe functions until they exec.
#[thread_local]
static CACHED_OS_TID_INVALID_AFTER_FORK: Cell<u32> = Cell::new(0);

// Assumes TIDs are unique between processes, which I only know is true for Redox.
fn os_tid_invalid_after_fork() -> u32 {
    // TODO: Coordinate better if using shared == PTHREAD_PROCESS_SHARED, with up to 2^32 separate
    // threads within possibly distinct processes, using the mutex. OS thread IDs on Redox are
    // pointer-sized, but relibc and POSIX uses int everywhere.
    
    let value = CACHED_OS_TID_INVALID_AFTER_FORK.get();

    if value == 0 {
        let tid = Sys::gettid();

        assert_ne!(tid, -1, "failed to obtain current thread ID");

        CACHED_OS_TID_INVALID_AFTER_FORK.set(tid as u32);

        tid as u32
    } else {
        value
    }
}
