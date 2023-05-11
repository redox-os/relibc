//! pthread.h implementation for Redox, following https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/pthread.h.html

use core::ptr::NonNull;
use core::cell::Cell;

use crate::platform::{self, Pal, Sys, types::*};
use crate::header::{sched::*, time::timespec};
use crate::pthread;

pub fn e(result: Result<(), pthread::Errno>) -> i32 {
    match result {
        Ok(()) => 0,
        Err(pthread::Errno(error)) => error,
    }
}

#[derive(Clone, Copy)]
pub(crate) struct RlctAttr {
    pub detachstate: c_uchar,
    pub inheritsched: c_uchar,
    pub schedpolicy: c_uchar,
    pub scope: c_uchar,
    pub guardsize: size_t,
    pub stacksize: size_t,
    pub stack: size_t,
    pub param: sched_param,
}

pub const PTHREAD_BARRIER_SERIAL_THREAD: c_int = -1;

pub const PTHREAD_CANCEL_ASYNCHRONOUS: c_int = 0;
pub const PTHREAD_CANCEL_ENABLE: c_int = 1;
pub const PTHREAD_CANCEL_DEFERRED: c_int = 2;
pub const PTHREAD_CANCEL_DISABLE: c_int = 3;
pub const PTHREAD_CANCELED: *mut c_void = (!0_usize) as *mut c_void;

pub const PTHREAD_CREATE_DETACHED: c_int = 0;
pub const PTHREAD_CREATE_JOINABLE: c_int = 1;

pub const PTHREAD_EXPLICIT_SCHED: c_int = 0;
pub const PTHREAD_INHERIT_SCHED: c_int = 1;

pub const PTHREAD_MUTEX_DEFAULT: c_int = 0;
pub const PTHREAD_MUTEX_ERRORCHECK: c_int = 1;
pub const PTHREAD_MUTEX_NORMAL: c_int = 2;
pub const PTHREAD_MUTEX_RECURSIVE: c_int = 3;

pub const PTHREAD_MUTEX_ROBUST: c_int = 0;
pub const PTHREAD_MUTEX_STALLED: c_int = 1;

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
    let attr = attr.cast::<RlctAttr>().as_ref();

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

#[no_mangle]
pub unsafe extern "C" fn pthread_getconcurrency() -> c_int {
    // Redox and Linux threads are 1:1, not M:N.
    1
}

#[no_mangle]
pub unsafe extern "C" fn pthread_getcpuclockid(thread: pthread_t, clock_out: *mut clockid_t) -> c_int {
    match pthread::get_cpu_clkid(&*thread.cast()) {
        Ok(clock) => {
            clock_out.write(clock);
            0
        }
        Err(pthread::Errno(error)) => error,
    }
}

#[no_mangle]
pub unsafe extern "C" fn pthread_getschedparam(thread: pthread_t, policy_out: *mut c_int, param_out: *mut sched_param) -> c_int {
    match pthread::get_sched_param(&*thread.cast()) {
        Ok((policy, param)) => {
            policy_out.write(policy);
            param_out.write(param);

            0
        }
        Err(pthread::Errno(error)) => error,
    }
}

pub mod tls;
pub use tls::*;

#[no_mangle]
pub unsafe extern "C" fn pthread_join(thread: pthread_t, retval: *mut *mut c_void) -> c_int {
    match pthread::join(&*thread.cast()) {
        Ok(pthread::Retval(ret)) => {
            if !retval.is_null() {
                core::ptr::write(retval, ret);
            }
            0
        }
        Err(pthread::Errno(error)) => error,
    }
}

pub mod mutex;
pub use self::mutex::*;

pub mod once;
pub use self::once::*;

pub mod rwlock;
pub use self::rwlock::*;

#[no_mangle]
pub unsafe extern "C" fn pthread_self() -> pthread_t {
    pthread::current_thread().unwrap_unchecked() as *const _ as *mut _
}
#[no_mangle]
pub unsafe extern "C" fn pthread_setcancelstate(state: c_int, oldstate: *mut c_int) -> c_int {
    match pthread::set_cancel_state(state) {
        Ok(old) => {
            oldstate.write(old);
            0
        }
        Err(pthread::Errno(error)) => error,
    }
}
#[no_mangle]
pub unsafe extern "C" fn pthread_setcanceltype(ty: c_int, oldty: *mut c_int) -> c_int {
    match pthread::set_cancel_type(ty) {
        Ok(old) => {
            oldty.write(old);
            0
        }
        Err(pthread::Errno(error)) => error,
    }
}

#[no_mangle]
pub extern "C" fn pthread_setconcurrency(concurrency: c_int) -> c_int {
    // Redox and Linux threads are 1:1, not M:N.
    0
}

#[no_mangle]
pub unsafe extern "C" fn pthread_setschedparam(thread: pthread_t, policy: c_int, param: *const sched_param) -> c_int {
    e(pthread::set_sched_param(&*thread.cast(), policy, &*param))
}
#[no_mangle]
pub unsafe extern "C" fn pthread_setschedprio(thread: pthread_t, prio: c_int) -> c_int {
    e(pthread::set_sched_priority(&*thread.cast(), prio))
}

pub mod spin;
pub use self::spin::*;

#[no_mangle]
pub unsafe extern "C" fn pthread_testcancel() {
    pthread::testcancel();
}

// Must be the same struct as defined in the pthread_cleanup_push macro.
#[repr(C)]
pub(crate) struct CleanupLinkedListEntry {
    routine: extern "C" fn(*mut c_void),
    arg: *mut c_void,
    prev: *const c_void,
}

#[thread_local]
pub(crate) static CLEANUP_LL_HEAD: Cell<*const CleanupLinkedListEntry> = Cell::new(core::ptr::null());

// TODO: unwind? setjmp/longjmp?

#[no_mangle]
pub unsafe extern "C" fn __relibc_internal_pthread_cleanup_push(new_entry: *mut c_void) {
    let new_entry = &mut *new_entry.cast::<CleanupLinkedListEntry>();

    new_entry.prev = CLEANUP_LL_HEAD.get().cast();
    CLEANUP_LL_HEAD.set(new_entry);
}
#[no_mangle]
pub unsafe extern "C" fn __relibc_internal_pthread_cleanup_pop(execute: c_int) {
    let prev_head = CLEANUP_LL_HEAD.get().read();
    CLEANUP_LL_HEAD.set(prev_head.prev.cast());

    if execute != 0 {
        (prev_head.routine)(prev_head.arg);
    }
}

pub(crate) unsafe fn run_destructor_stack() {
    let mut ptr = CLEANUP_LL_HEAD.get();

    while !ptr.is_null() {
        let entry = ptr.read();
        ptr = entry.prev.cast();

        (entry.routine)(entry.arg);
    }
}
