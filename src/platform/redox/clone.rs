use core::mem::size_of;

use syscall::{
    data::Map,
    error::Result,
    flag::{MapFlags, O_CLOEXEC},
    SetSighandlerData, SIGCONT,
};

use crate::sync::rwlock::Rwlock;

use redox_rt::{proc::FdGuard, signal::sighandler_function};

pub use redox_rt::proc::*;

static CLONE_LOCK: Rwlock = Rwlock::new(crate::pthread::Pshared::Private);

struct Guard;
impl Drop for Guard {
    fn drop(&mut self) {
        CLONE_LOCK.unlock()
    }
}

pub fn rdlock() -> impl Drop {
    CLONE_LOCK.acquire_read_lock(None);

    Guard
}
pub fn wrlock() -> impl Drop {
    CLONE_LOCK.acquire_write_lock(None);

    Guard
}
pub use redox_rt::thread::*;
