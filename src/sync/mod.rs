pub mod barrier;
pub mod cond;
// TODO: Merge with pthread_mutex
pub mod mutex;

pub mod once;
pub mod pthread_mutex;
pub mod rwlock;
pub mod semaphore;
pub mod waitval;

pub use self::{
    mutex::{Mutex, MutexGuard},
    once::Once,
    semaphore::Semaphore,
};

use crate::{
    header::time::timespec,
    platform::{types::*, Pal, Sys},
};
use core::{
    mem::MaybeUninit,
    ops::Deref,
    sync::atomic::{self, AtomicI32, AtomicU32, AtomicI32 as AtomicInt},
};

const FUTEX_WAIT: c_int = 0;
const FUTEX_WAKE: c_int = 1;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AttemptStatus {
    Desired,
    Waiting,
    Other,
}

pub trait FutexTy {
    fn conv(self) -> i32;
}
pub trait FutexAtomicTy {
    type Ty: FutexTy;

    fn ptr(&self) -> *mut Self::Ty;
}
impl FutexTy for u32 {
    fn conv(self) -> i32 {
        self as i32
    }
}
impl FutexTy for i32 {
    fn conv(self) -> i32 {
        self
    }
}
impl FutexAtomicTy for AtomicU32 {
    type Ty = u32;

    fn ptr(&self) -> *mut u32 {
        // TODO: Change when Redox's toolchain is updated. This is not about targets, but compiler
        // versions!

        #[cfg(target_os = "redox")]
        return AtomicU32::as_ptr(self);

        #[cfg(target_os = "linux")]
        return AtomicU32::as_mut_ptr(self);
    }
}
impl FutexAtomicTy for AtomicI32 {
    type Ty = i32;

    fn ptr(&self) -> *mut i32 {
        // TODO
        #[cfg(target_os = "redox")]
        return AtomicI32::as_ptr(self);

        #[cfg(target_os = "linux")]
        return AtomicI32::as_mut_ptr(self);
    }
}

pub unsafe fn futex_wake_ptr(ptr: *mut impl FutexTy, n: i32) -> usize {
    // TODO: unwrap_unchecked?
    Sys::futex(ptr.cast(), FUTEX_WAKE, n, 0).unwrap() as usize
}
pub unsafe fn futex_wait_ptr<T: FutexTy>(ptr: *mut T, value: T, timeout_opt: Option<&timespec>) -> bool {
    // TODO: unwrap_unchecked?
    Sys::futex(ptr.cast(), FUTEX_WAIT, value.conv(), timeout_opt.map_or(0, |t| t as *const _ as usize)) == Ok(0)
}
pub fn futex_wake(atomic: &impl FutexAtomicTy, n: i32) -> usize {
    unsafe { futex_wake_ptr(atomic.ptr(), n) }
}
pub fn futex_wait<T: FutexAtomicTy>(atomic: &T, value: T::Ty, timeout_opt: Option<&timespec>) -> bool {
    unsafe { futex_wait_ptr(atomic.ptr(), value, timeout_opt) }
}

pub fn rttime() -> timespec {
    unsafe {
        let mut time = MaybeUninit::uninit();

        // TODO: Handle error
        Sys::clock_gettime(crate::header::time::CLOCK_REALTIME, time.as_mut_ptr());

        time.assume_init()
    }
}

pub fn wait_until_generic<F1, F2>(word: &AtomicInt, attempt: F1, mark_long: F2, long: c_int)
where
    F1: Fn(&AtomicInt) -> AttemptStatus,
    F2: Fn(&AtomicInt) -> AttemptStatus,
{
    // First, try spinning for really short durations
    for _ in 0..999 {
        atomic::spin_loop_hint();
        if attempt(word) == AttemptStatus::Desired {
            return;
        }
    }

    // One last attempt, to initiate "previous"
    let mut previous = attempt(word);

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
            mark_long(word) != AttemptStatus::Desired
        {
            futex_wait(word, long, None);
        }

        previous = attempt(word);
    }
}

/// Convenient wrapper around the "futex" system call for
/// synchronization implementations
#[repr(C)]
pub(crate) struct AtomicLock {
    pub(crate) atomic: AtomicInt,
}
impl AtomicLock {
    pub const fn new(value: c_int) -> Self {
        Self {
            atomic: AtomicInt::new(value),
        }
    }
    pub fn notify_one(&self) {
        futex_wake(&self.atomic, 1);
    }
    pub fn notify_all(&self) {
        futex_wake(&self.atomic, i32::MAX);
    }
    pub fn wait_if(&self, value: c_int, timeout_opt: Option<&timespec>) {
        self.wait_if_raw(value, timeout_opt);
    }
    pub fn wait_if_raw(&self, value: c_int, timeout_opt: Option<&timespec>) -> bool {
        futex_wait(&self.atomic, value, timeout_opt)
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
        wait_until_generic(&self.atomic, attempt, mark_long, long)
    }
}
impl Deref for AtomicLock {
    type Target = AtomicInt;

    fn deref(&self) -> &Self::Target {
        &self.atomic
    }
}
