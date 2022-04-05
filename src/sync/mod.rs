pub mod mutex;
pub mod once;
pub mod semaphore;

pub use self::{
    mutex::{Mutex, MutexGuard},
    once::Once,
    semaphore::Semaphore,
};

use crate::header::time::timespec;
use crate::platform::{types::*, Pal, Sys};
use core::{
    cell::UnsafeCell,
    ops::Deref,
    sync::atomic::{self, AtomicI32 as AtomicInt},
};

const FUTEX_WAIT: c_int = 0;
const FUTEX_WAKE: c_int = 1;

#[derive(Clone, Copy, PartialEq, Eq)]
enum AttemptStatus {
    Desired,
    Waiting,
    Other,
}

/// Convenient wrapper around the "futex" system call for
/// synchronization implementations
struct AtomicLock {
    atomic: UnsafeCell<AtomicInt>,
}
impl AtomicLock {
    pub const fn new(value: c_int) -> Self {
        Self {
            atomic: UnsafeCell::new(AtomicInt::new(value)),
        }
    }
    pub fn notify_one(&self) {
        Sys::futex(unsafe { &mut *self.atomic.get() }.get_mut(), FUTEX_WAKE, 1, 0);
    }
    pub fn notify_all(&self) {
        Sys::futex(
            unsafe { &mut *self.atomic.get() }.get_mut(),
            FUTEX_WAKE,
            c_int::max_value(),
            0
        );
    }
    pub fn wait_if(&self, value: c_int, timeout_opt: Option<&timespec>) {
        Sys::futex(
            unsafe { &mut *self.atomic.get() }.get_mut(),
            FUTEX_WAIT,
            value,
            timeout_opt.map_or(0, |timeout| timeout as *const timespec as usize)
        );
    }

    /// A general way to efficiently wait for what might be a long time, using two closures:
    ///
    /// - `attempt` = Attempt to modify the atomic value to any
    /// desired state.
    /// - `mark_long` = Attempt to modify the atomic value to sign
    /// that it want's to get notified when waiting is done.
    ///
    /// Both of these closures are allowed to spuriously give a
    /// non-success return value, they are used only as optimization
    /// hints. However, what counts as a "desired value" may differ
    /// per closure. Therefore, `mark_long` can notify a value as
    /// "desired" in order to get `attempt` retried immediately.
    ///
    /// The `long` parameter is the only one which actually cares
    /// about the specific value of your atomics. This is needed
    /// because it needs to pass this to the futex system call in
    /// order to avoid race conditions where the atomic could be
    /// modified to the desired value before the call is complete and
    /// we receive the wakeup notification.
    pub fn wait_until<F1, F2>(&self, attempt: F1, mark_long: F2, long: c_int)
    where
        F1: Fn(&AtomicInt) -> AttemptStatus,
        F2: Fn(&AtomicInt) -> AttemptStatus,
    {
        // First, try spinning for really short durations
        for _ in 0..999 {
            atomic::spin_loop_hint();
            if attempt(self) == AttemptStatus::Desired {
                return;
            }
        }

        // One last attempt, to initiate "previous"
        let mut previous = attempt(self);

        // Ok, that seems to take quite some time. Let's go into a
        // longer, more patient, wait.
        loop {
            if previous == AttemptStatus::Desired {
                return;
            }

            if
            // If we or somebody else already initiated a long
            // wait, OR
            previous == AttemptStatus::Waiting ||
                // Otherwise, unless our attempt to initiate a long
                // wait informed us that we might be done waiting
                mark_long(self) != AttemptStatus::Desired
            {
                self.wait_if(long, None);
            }

            previous = attempt(self);
        }
    }
}
impl Deref for AtomicLock {
    type Target = AtomicInt;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.atomic.get() }
    }
}
