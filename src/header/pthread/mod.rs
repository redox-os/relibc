//! pthread.h implementation for Redox, following https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/pthread.h.html

use core::ptr::NonNull;

use crate::platform::{self, Pal, Sys, types::*};
use crate::header::{sched::*, time::timespec};
use crate::pthread;

pub const PTHREAD_BARRIER_SERIAL_THREAD: c_int = 1;

pub const PTHREAD_CANCEL_ASYNCHRONOUS: c_int = 0;
pub const PTHREAD_CANCEL_ENABLE: c_int = 0;
pub const PTHREAD_CANCEL_DEFERRED: c_int = 0;
pub const PTHREAD_CANCEL_DISABLE: c_int = 0;
pub const PTHREAD_CANCELED: *mut c_void = core::ptr::null_mut();

pub const PTHREAD_CREATE_DETACHED: c_int = 0;
pub const PTHREAD_CREATE_JOINABLE: c_int = 1;

pub const PTHREAD_EXPLICIT_SCHED: c_int = 0;
pub const PTHREAD_INHERIT_SCHED: c_int = 1;

pub const PTHREAD_MUTEX_DEFAULT: c_int = 0;
pub const PTHREAD_MUTEX_ERRORCHECK: c_int = 0;
pub const PTHREAD_MUTEX_NORMAL: c_int = 0;
pub const PTHREAD_MUTEX_RECURSIVE: c_int = 0;
pub const PTHREAD_MUTEX_ROBUST: c_int = 0;
pub const PTHREAD_MUTEX_STALLED: c_int = 0;

pub const PTHREAD_PRIO_INHERIT: c_int = 0;

pub const PTHREAD_PRIO_NONE: c_int = 0;

pub const PTHREAD_PRIO_PROTECT: c_int = 0;

pub const PTHREAD_PROCESS_SHARED: c_int = 0;
pub const PTHREAD_PROCESS_PRIVATE: c_int = 1;

pub const PTHREAD_SCOPE_PROCESS: c_int = 0;
pub const PTHREAD_SCOPE_SYSTEM: c_int = 1;

pub mod attr;
pub use self::attr::*;

pub mod barrier;
pub use self::barrier::*;

#[no_mangle]
pub unsafe extern "C" fn pthread_cancel(thread: pthread_t) -> c_int {
    match pthread::cancel(&*thread.cast()) {
        Ok(()) => 0,
        Err(pthread::Errno(error)) => error,
    }
}

pub mod cond;
pub use self::cond::*;

#[no_mangle]
pub unsafe extern "C" fn pthread_create(pthread: *mut pthread_t, attr: *const pthread_attr_t, start_routine: extern "C" fn(arg: *mut c_void) -> *mut c_void, arg: *mut c_void) -> c_int {
    let attr = NonNull::new(attr as *mut _).map(|n| n.as_ref());

    match pthread::create(attr, start_routine, arg) {
        Ok(ptr) => {
            core::ptr::write(pthread, ptr);
            0
        }
        Err(pthread::Errno(code)) => code,
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_detach(pthread: pthread_t) -> c_int {
    match pthread::detach(&*pthread.cast()) {
        Ok(()) => 0,
        Err(pthread::Errno(errno)) => errno,
    }
}

#[no_mangle]
pub extern "C" fn pthread_equal(pthread1: pthread_t, pthread2: pthread_t) -> c_int {
    core::ptr::eq(pthread1, pthread2).into()
}

#[no_mangle]
pub unsafe extern "C" fn pthread_exit(retval: *mut c_void) -> ! {
    pthread::exit_current_thread(pthread::Retval(retval))
}

// #[no_mangle]
pub extern "C" fn pthread_getconcurrency() -> c_int {
    todo!()
}

// #[no_mangle]
pub extern "C" fn pthread_getcpuclockid(thread: pthread_t, clock: *mut clockid_t) -> c_int {
    todo!()
}

// #[no_mangle]
pub extern "C" fn pthread_getschedparam(thread: pthread_t, policy: *mut clockid_t, param: *mut sched_param) -> c_int {
    todo!()
}

// #[no_mangle]
pub extern "C" fn pthread_getspecific(key: pthread_key_t) -> *mut c_void {
    todo!()
}

#[no_mangle]
pub unsafe extern "C" fn pthread_join(thread: pthread_t, retval: *mut *mut c_void) -> c_int {
    match pthread::join(&*thread.cast()) {
        Ok(pthread::Retval(ret)) => {
            core::ptr::write(retval, ret);
            0
        }
        Err(pthread::Errno(error)) => error,
    }
}

// #[no_mangle]
pub extern "C" fn pthread_key_create(key: *mut pthread_key_t, destructor: extern "C" fn(value: *mut c_void)) -> c_int {
    todo!()
}

// #[no_mangle]
pub extern "C" fn pthread_key_delete(key: pthread_key_t) -> c_int {
    todo!()
}

pub mod mutex;
pub use self::mutex::*;

pub mod once;
pub use self::once::*;

pub mod rwlock;
pub use self::rwlock::*;

pub unsafe extern "C" fn pthread_self() -> pthread_t {
    pthread::current_thread().unwrap_unchecked() as *const _ as *mut _
}
pub extern "C" fn pthread_setcancelstate(state: c_int, oldstate: *mut c_int) -> c_int {
    todo!();
}
pub extern "C" fn pthread_setcanceltype(ty: c_int, oldty: *mut c_int) -> c_int {
    todo!();
}

pub extern "C" fn pthread_setconcurrency(concurrency: c_int) -> c_int {
    todo!();
}

pub extern "C" fn pthread_setschedparam(thread: pthread_t, policy: c_int, param: *const sched_param) -> c_int {
    todo!();
}
pub extern "C" fn pthread_setschedprio(thread: pthread_t, prio: c_int) -> c_int {
    todo!();
}
pub extern "C" fn pthread_setspecific(key: pthread_key_t, value: *const c_void) -> c_int {
    todo!();
}

pub mod spin;
pub use self::spin::*;

pub unsafe extern "C" fn pthread_testcancel() {
    pthread::testcancel();
}

// pthread_cleanup_pop()
// pthread_cleanup_push()
