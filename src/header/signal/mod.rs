//! `signal.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/signal.h.html>.

use core::{mem, ptr};

use cbitset::BitSet;

#[cfg(target_os = "redox")]
use crate::platform::types::pthread_attr_t;
use crate::{
    error::{Errno, ResultExt},
    header::{bits_sigset_t::sigset_t, errno, time::timespec},
    platform::{
        self, ERRNO, Pal, PalSignal, Sys,
        types::{c_char, c_int, c_ulonglong, c_void, pid_t, pthread_t, size_t, uid_t},
    },
};

pub mod constants;
pub use self::sys::*;
pub use constants::*;

use super::{
    errno::EFAULT,
    stdio::{fprintf, stderr},
};

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod sys;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;

type SigSet = BitSet<[u64; 1]>;

/// cbindgen:ignore
/// Request for default signal handling.
pub(crate) const SIG_DFL: usize = 0;
/// cbindgen:ignore
/// Request that signal be ignored.
pub(crate) const SIG_IGN: usize = 1;
/// cbindgen:ignore
/// Return value of `signal()` in case of error.
pub(crate) const SIG_ERR: isize = -1;
/// cbindgen:ignore
/// Obsolete in issue 7, removed in issue 8.
/// Request that signal be held.
pub(crate) const SIG_HOLD: isize = 2;

/// The resulting set is the union of the current set and the signal set
/// pointed to by the argument `set`.
pub const SIG_BLOCK: c_int = 0;
/// The resulting set is the intersection of the current set and the compliment
/// of the signal set pointed to by the argument `set`.
pub const SIG_UNBLOCK: c_int = 1;
/// The resulting set is the signal set pointed to by the argument `set`.
pub const SIG_SETMASK: c_int = 2;

/// A queued signal, with an application-defined value, is generated when the
/// event of interest occurs.
pub const SIGEV_SIGNAL: c_int = 0;
/// No asynchronous notification is delivered when the event of interest
/// occurs.
pub const SIGEV_NONE: c_int = 1;
/// A notification function is called to perform notification.
pub const SIGEV_THREAD: c_int = 2;

/// cbindgen:ignore
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/signal.h.html>.
///
/// # Implementation
/// This struct in Rust is missing the `sa_sigaction` field. The stucture in
/// cbindgen uses a union to combine `sa_handler` and `sa_sigaction`. POSIX
/// states that both fields shall not be used simultaneously.
#[repr(C)]
#[derive(Clone, Debug)]
pub struct sigaction {
    /// Pointer to a signal-catching function or one of the `SIG_IGN` or
    /// `SIG_DFL`.
    pub sa_handler: Option<extern "C" fn(c_int)>,
    /// Special flags.
    pub sa_flags: c_int,
    /// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/sigaction.2.html>.
    ///
    /// Not intended for application use. A sigaction wrapper function is
    /// intended to use this to store the location of the trampoline code and
    /// setting the `SA_RESTORER` flag in `sa_flags`.
    pub sa_restorer: Option<unsafe extern "C" fn()>,
    /// Set of signals to be blocked during execution of the signal handling
    /// function.
    pub sa_mask: sigset_t,
}

#[repr(C)]
#[derive(Clone)]
pub struct sigaltstack {
    /// Stack base or pointer.
    pub ss_sp: *mut c_void,
    /// Flags,
    pub ss_flags: c_int,
    /// Stack size.
    pub ss_size: size_t,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/signal.h.html>.
#[repr(C)]
#[derive(Clone)]
#[cfg(not(target_os = "linux"))]
pub struct sigevent {
    /// Signal value.
    pub sigev_value: sigval,
    /// Signal number.
    pub sigev_signo: c_int,
    /// Notification type.
    pub sigev_notify: c_int,
    /// Notification function.
    pub sigev_notify_function: Option<extern "C" fn(sigval)>,
    /// Notification attributes.
    pub sigev_notify_attributes: *mut pthread_attr_t,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/signal.h.html>.
///
/// # Implementation
/// Must match with signature from libc.
/// See <https://docs.rs/libc/0.2.186/src/libc/unix/linux_like/mod.rs.html#300-322>.
#[repr(C)]
#[derive(Clone)]
#[cfg(target_os = "linux")]
pub struct sigevent {
    /// Signal value.
    pub sigev_value: sigval,
    /// Signal number.
    pub sigev_signo: c_int,
    /// Notification type.
    pub sigev_notify: c_int,
    // Actually a union.  We only expose sigev_notify_thread_id because it's
    // the most useful member
    pub sigev_notify_thread_id: c_int,
    #[cfg(target_pointer_width = "64")]
    __unused1: [c_int; 11],
    #[cfg(target_pointer_width = "32")]
    __unused1: [c_int; 12],
}

// FIXME: This struct is wrong on Linux
#[repr(C)]
#[derive(Clone, Copy)]
pub struct siginfo {
    /// Signal number.
    pub si_signo: c_int,
    /// If non-zero, an errno value associated with this signal.
    pub si_errno: c_int,
    /// Signal code.
    pub si_code: c_int,
    /// Sending process ID.
    pub si_pid: pid_t,
    /// Real user ID of sending process.
    pub si_uid: uid_t,
    /// Address that caused fault.
    pub si_addr: *mut c_void,
    /// Exit value or signal.
    pub si_status: c_int,
    /// Signal value.
    pub si_value: sigval,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/signal.h.html>.
///
/// Signal value.
#[derive(Clone, Copy)]
#[repr(C)]
pub union sigval {
    /// Integer signal value.
    pub sival_int: c_int,
    /// Pointer signal value.
    pub sival_ptr: *mut c_void,
}

/// cbindgen:ignore
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/signal.h.html>.
pub type siginfo_t = siginfo;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/signal.h.html>
pub type stack_t = sigaltstack;

//NOTE for the following two functions, to see why they're implemented slightly differently from their intended behavior, read
//     https://git.musl-libc.org/cgit/musl/commit/?id=583e55122e767b1586286a0d9c35e2a4027998ab
#[unsafe(no_mangle)]
unsafe extern "C" fn __sigsetjmp_tail(jb: *mut c_ulonglong, ret: c_int) -> c_int {
    let set = jb.wrapping_add(9);
    if ret > 0 {
        unsafe { sigprocmask(SIG_SETMASK, set, ptr::null_mut()) };
    } else {
        unsafe { sigprocmask(SIG_SETMASK, ptr::null_mut(), set) };
    }
    ret
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/kill.html>.
///
/// Sends a signal to a process or a group of processes specified by `pid`.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error. No signal is sent if failed.
#[unsafe(no_mangle)]
pub extern "C" fn kill(pid: pid_t, sig: c_int) -> c_int {
    Sys::kill(pid, sig).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigqueue.html>.
///
/// Causes the signal specified by `signo` to be sent with the value specified
/// by `value` to the process specified by `pid`.
///
/// Upon success, the specified signal shall have been queued, and returns `0`.
/// Upon error, returns `-1` and sets errno to indicate the error.
#[unsafe(no_mangle)]
pub extern "C" fn sigqueue(pid: pid_t, signo: c_int, value: sigval) -> c_int {
    Sys::sigqueue(pid, signo, value)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/killpg.html>.
///
/// Sends the signali specified by `sig` to the process group specified by
/// `pgrp`.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error. No signal is sent if failed.
#[unsafe(no_mangle)]
pub extern "C" fn killpg(pgrp: pid_t, sig: c_int) -> c_int {
    Sys::killpg(pgrp, sig).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_kill.html>.
///
/// Requests that a signal be delivered to the specified thread. It shall not
/// be an error is `thread` is a zombie thread.
///
/// Upon success, returns `0`. Upon failure, returns an error number and does
/// not send the signal.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_kill(thread: pthread_t, sig: c_int) -> c_int {
    let os_tid = {
        let pthread = unsafe { &*(thread as *const crate::pthread::Pthread) };
        unsafe { pthread.os_tid.get().read() }
    };
    crate::header::pthread::e(unsafe { Sys::rlct_kill(os_tid, sig as usize) })
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_sigmask.html>.
///
/// Examines or changes (or both) the calling thread's signal mask.
///
/// Upon success, returns `0`. Upon failure, returns an error number.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_sigmask(
    how: c_int,
    set: *const sigset_t,
    oldset: *mut sigset_t,
) -> c_int {
    // On Linux and Redox, pthread_sigmask and sigprocmask are equivalent
    if unsafe { sigprocmask(how, set, oldset) } == 0 {
        0
    } else {
        //TODO: Fix race
        platform::ERRNO.get()
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/raise.html>.
///
/// Sends the signal `sig` to the executing thread or process. If a signal
/// handler is called, this function shall not return until after the signal
/// handler does.
///
/// Upon success, returns `0`. Upon failure, returns a non-zero value and sets
/// errno to indicate the error.
#[unsafe(no_mangle)]
pub extern "C" fn raise(sig: c_int) -> c_int {
    Sys::raise(sig).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigaction.html>.
///
/// Allows the calling process to examine and/or specify the action to be
/// associated with a specific signal.
///
/// Upon success, returns `0`. Upon failure, returns `-1`, sets errno to
/// indicate the error, and no new signal-catching function shall be installed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigaction(
    sig: c_int,
    act: *const sigaction,
    oact: *mut sigaction,
) -> c_int {
    Sys::sigaction(sig, unsafe { act.as_ref() }, unsafe { oact.as_mut() })
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigaddset.html>.
///
/// Adds the individual signal specified by `signo` to the signal set pointed
/// to by `set`.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error.
///
/// # Safety
/// The `sigset_t` pointed to by `set` must be initialized by `sigemptyset()`
/// or `sigfillset()` before calling this function or undefined behaviour will
/// occur.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigaddset(set: *mut sigset_t, signo: c_int) -> c_int {
    if signo <= 0 || signo as usize > NSIG.max(SIGRTMAX)
    /* TODO */
    {
        platform::ERRNO.set(errno::EINVAL);
        return -1;
    }

    if let Some(set) = unsafe { (set.cast::<SigSet>()).as_mut() } {
        set.insert(signo as usize - 1); // 0-indexed usize, please!
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigaltstack.html>.
///
/// Allows a process to define and examine the state of an alternate stack for
/// signal handlers for the current thread.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error.
///
/// # Safety
/// Use of this function by library threads that are not bound to
/// kernel-scheduled entities results in undefined behaviour.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigaltstack(ss: *const stack_t, old_ss: *mut stack_t) -> c_int {
    unsafe {
        Sys::sigaltstack(ss.as_ref(), old_ss.as_mut())
            .map(|()| 0)
            .or_minus_one_errno()
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigdelset.html>.
///
/// Deletes the individual signal specified by `signo` to the signal set
/// pointed to by `set`.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error.
///
/// # Safety
/// The `sigset_t` pointed to by `set` must be initialized by `sigemptyset()`
/// or `sigfillset()` before calling this function or undefined behaviour will
/// occur.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigdelset(set: *mut sigset_t, signo: c_int) -> c_int {
    if signo <= 0 || signo as usize > NSIG.max(SIGRTMAX)
    /* TODO */
    {
        platform::ERRNO.set(errno::EINVAL);
        return -1;
    }

    if let Some(set) = unsafe { (set.cast::<SigSet>()).as_mut() } {
        set.remove(signo as usize - 1); // 0-indexed usize, please!
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigemptyset.html>.
///
/// Initializes the signal set pointed to by `set`, such that all signals
/// defined in POSIX.1-2024 are excluded.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigemptyset(set: *mut sigset_t) -> c_int {
    if let Some(set) = unsafe { (set.cast::<SigSet>()).as_mut() } {
        set.clear();
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigfillset.html>.
///
/// Initializes the signal set pointed to by `set`, such that all signals
/// defined in POSIX.1-2024 are included.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigfillset(set: *mut sigset_t) -> c_int {
    if let Some(set) = unsafe { (set.cast::<SigSet>()).as_mut() } {
        set.fill(.., true);
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/sighold.html>.
///
/// Adds `sig` to the signal mask of the calling process.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error.
///
/// # Deprecated
/// Present in issue 7. Removed in issue 8.
///
/// Use of this function is unspecified in a multi-threaded process.
///
/// `pthread_sigmask()` or `sigprocmask()` should be used instead.
///
/// # Implementation
/// Calls `sigprocmask()` internally.
#[deprecated]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sighold(sig: c_int) -> c_int {
    let mut pset = mem::MaybeUninit::<sigset_t>::uninit();
    unsafe { sigemptyset(pset.as_mut_ptr()) };
    let mut set = unsafe { pset.assume_init() };
    if unsafe { sigaddset(&raw mut set, sig) } < 0 {
        return -1;
    }
    unsafe { sigprocmask(SIG_BLOCK, &raw const set, ptr::null_mut()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/sighold.html>.
///
/// Sets the disposition of `sig` to `SIG_IGN`.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error.
///
/// # Deprecated
/// Present in issue 7. Removed in issue 8.
///
/// Use of this function is unspecified in a multi-threaded process.
///
/// `sigaction()` should be used instead.
///
/// # Implementation
/// Calls `sigaction()` internally.
#[deprecated]
#[expect(clippy::missing_transmute_annotations, reason = "too verbose")]
#[unsafe(no_mangle)]
pub extern "C" fn sigignore(sig: c_int) -> c_int {
    let mut psa = mem::MaybeUninit::<sigaction>::uninit();
    unsafe { sigemptyset(&raw mut (*psa.as_mut_ptr()).sa_mask) };
    let mut sa = unsafe { psa.assume_init() };
    sa.sa_handler = unsafe { mem::transmute(SIG_IGN) };
    sa.sa_flags = 0;
    unsafe { sigaction(sig, &raw const sa, ptr::null_mut()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/siginterrupt.html>.
///
/// Changes the restart behaviour when a function is interrupted by the
/// specified signal.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error.
///
/// # Deprecated
/// Marked obsolescent in issue 7. Removed in issue 8.
///
/// Should use `sigaction()` with the `SA_RESTART` flag instead.
///
/// # Implementation
/// Internally uses `sigaction()` with the `SA_RESTART` flag.
#[deprecated]
#[unsafe(no_mangle)]
pub extern "C" fn siginterrupt(sig: c_int, flag: c_int) -> c_int {
    let mut psa = mem::MaybeUninit::<sigaction>::uninit();
    unsafe { sigaction(sig, ptr::null_mut(), psa.as_mut_ptr()) };
    let mut sa = unsafe { psa.assume_init() };
    if flag != 0 {
        sa.sa_flags &= !SA_RESTART as c_int;
    } else {
        sa.sa_flags |= SA_RESTART as c_int;
    }

    unsafe { sigaction(sig, &raw const sa, ptr::null_mut()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigismember.html>.
///
/// Tests whether the signal specified by `signo` is a member of the set
/// pointed to by `set`.
///
/// Upon success, return `1` if the specified signal is a member of the
/// specified set, or `0` if it is not. Upon failure, returns `-1` and sets
/// errno to indicate the error.
///
/// # Safety
/// The `sigset_t` pointed to by `set` must be initialized by `sigemptyset()`
/// or `sigfillset()` before calling this function or undefined behaviour will
/// occur.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigismember(set: *const sigset_t, signo: c_int) -> c_int {
    if signo <= 0 || signo as usize > NSIG.max(SIGRTMAX)
    /* TODO */
    {
        platform::ERRNO.set(errno::EINVAL);
        return -1;
    }

    if let Some(set) = unsafe { (set as *mut SigSet).as_mut() }
        && set.contains(signo as usize - 1)
    {
        return 1;
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/signal.html>.
///
/// Chooses one of three ways in which receipt of the signal number `sig` is to
/// be subsequently handled.
///
/// Upon success, returns the value of `func` for the most recent call to
/// `signal()` for the specified signal `sig`.Upon failure, returns `SIG_ERR`
/// and a positive value shall be stored in errno.
#[expect(clippy::missing_transmute_annotations, reason = "too verbose")]
#[unsafe(no_mangle)]
pub extern "C" fn signal(
    sig: c_int,
    func: Option<extern "C" fn(c_int)>,
) -> Option<extern "C" fn(c_int)> {
    let sa = sigaction {
        sa_handler: func,
        sa_flags: SA_RESTART as _,
        sa_restorer: None, // set by platform if applicable
        sa_mask: sigset_t::default(),
    };
    let mut old_sa = mem::MaybeUninit::uninit();
    if unsafe { sigaction(sig, &raw const sa, old_sa.as_mut_ptr()) } < 0 {
        return unsafe { mem::transmute(SIG_ERR) };
    }
    unsafe { old_sa.assume_init() }.sa_handler
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/sighold.html>.
///
/// Removes `sig` from the signal mask of the calling process and suspend the
/// calling process until a signal is received.
///
/// Suspends execution of the thread until a signal is received, whereupon it
/// shall return `-1` and set errno to `EINTR`.
///
/// # Deprecated
/// Present in issue 7. Removed in issue 8.
///
/// Use of this function is unspecified in a multi-threaded process.
///
/// `sigsuspend()` should be used instead.
///
/// # Implementation
/// Calls `sigsuspend()` internally.
#[deprecated]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigpause(sig: c_int) -> c_int {
    let mut pset = mem::MaybeUninit::<sigset_t>::uninit();
    unsafe { sigprocmask(0, ptr::null_mut(), pset.as_mut_ptr()) };
    let mut set = unsafe { pset.assume_init() };
    if unsafe { sigdelset(&raw mut set, sig) } == -1 {
        return -1;
    }
    unsafe { sigsuspend(&raw const set) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigpending.html>.
///
/// Stores, in the location referenced by the `set` argument, the set of
/// signals that are blocked from delivery to the calling thread and that are
/// pending on the process or the calling thread.
///
/// Upon success, returns `0`. Upon failure, returns `-1` and sets errno to
/// indicate the error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigpending(set: *mut sigset_t) -> c_int {
    (|| Sys::sigpending(unsafe { set.as_mut().ok_or(Errno(EFAULT)) }?))()
        .map(|()| 0)
        .or_minus_one_errno()
}

// TODO: Double-check this mask.
// This prevents the application from blocking the two signals SIGRTMIN - 1 and SIGRTMIN - 2 which
// are (at least meant to be) used internally for timers and pthread cancellation. On Linux this is
// 32 and 33 (same as NPTL reserves), whereas this on Redox is 33 and 34 (TODO: could this be
// changed to 32 and 33 for Redox too, since there's currently no support for "sigqueue" targeting
// specific threads).
const RLCT_SIGNAL_MASK: sigset_t = (1 << ((SIGRTMIN - 1) - 1)) | (1 << ((SIGRTMIN - 2) - 1));

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigprocmask.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigprocmask(
    how: c_int,
    set: *const sigset_t,
    oset: *mut sigset_t,
) -> c_int {
    (|| {
        let set = unsafe { set.as_ref().map(|&block| block & !RLCT_SIGNAL_MASK) };
        let mut oset = unsafe { oset.as_mut() };

        Sys::sigprocmask(
            how,
            set.as_ref(),
            oset.as_deref_mut(), // as_deref_mut for lifetime reasons
        )?;

        if let Some(oset) = oset {
            *oset &= !RLCT_SIGNAL_MASK;
        }

        Ok(0)
    })()
    .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/sighold.html>.
///
/// Present in issue 7. Removed in issue 8.
///
/// Use of this function is unspecified in a multi-threaded process.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigrelse(sig: c_int) -> c_int {
    let mut pset = mem::MaybeUninit::<sigset_t>::uninit();
    unsafe { sigemptyset(pset.as_mut_ptr()) };
    let mut set = unsafe { pset.assume_init() };
    if unsafe { sigaddset(&raw mut set, sig) } < 0 {
        return -1;
    }
    unsafe { sigprocmask(SIG_UNBLOCK, &raw const set, ptr::null_mut()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/sighold.html>.
///
/// Present in issue 7. Removed in issue 8.
///
/// Use of this function is unspecified in a multi-threaded process.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigset(
    sig: c_int,
    func: Option<extern "C" fn(c_int)>,
) -> Option<extern "C" fn(c_int)> {
    let mut old_sa = mem::MaybeUninit::uninit();
    let mut pset = mem::MaybeUninit::<sigset_t>::uninit();
    let sig_hold: Option<extern "C" fn(c_int)> = unsafe { mem::transmute(SIG_HOLD) };
    let sig_err: Option<extern "C" fn(c_int)> = unsafe { mem::transmute(SIG_ERR) };
    unsafe { sigemptyset(pset.as_mut_ptr()) };
    let mut set = unsafe { pset.assume_init() };
    if unsafe { sigaddset(&raw mut set, sig) } < 0 {
        return sig_err;
    } else {
        let is_equal = {
            match (func, sig_hold) {
                (None, None) => true,
                (Some(_), None) | (None, Some(_)) => false,
                (Some(f), Some(sh)) => ptr::fn_addr_eq(f, sh),
            }
        };
        if is_equal {
            if unsafe { sigaction(sig, ptr::null_mut(), old_sa.as_mut_ptr()) } < 0
                || unsafe { sigprocmask(SIG_BLOCK, &raw const set, &raw mut set) } < 0
            {
                return sig_err;
            }
        } else {
            let mut sa = sigaction {
                sa_handler: func,
                sa_flags: c_int::from(0),
                sa_restorer: None, // set by platform if applicable
                sa_mask: sigset_t::default(),
            };
            unsafe { sigemptyset(&raw mut sa.sa_mask) };
            if unsafe { sigaction(sig, &raw const sa, old_sa.as_mut_ptr()) } < 0
                || unsafe { sigprocmask(SIG_UNBLOCK, &raw const set, &raw mut set) } < 0
            {
                return sig_err;
            }
        }
    }
    if unsafe { sigismember(&raw const set, sig) } == 1 {
        return sig_hold;
    }
    unsafe { old_sa.assume_init().sa_handler }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigsuspend.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigsuspend(sigmask: *const sigset_t) -> c_int {
    Err(Sys::sigsuspend(unsafe { &*sigmask })).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigwait.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigwait(set: *const sigset_t, sig: *mut c_int) -> c_int {
    let mut pinfo = mem::MaybeUninit::<siginfo_t>::uninit();
    if unsafe { sigtimedwait(set, pinfo.as_mut_ptr(), ptr::null_mut()) } < 0 {
        return -1;
    }
    let info = unsafe { pinfo.assume_init() };
    unsafe { (*sig) = info.si_signo };
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigtimedwait.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigtimedwait(
    set: *const sigset_t,
    // s/siginfo_t/siginfo due to https://github.com/mozilla/cbindgen/issues/621
    sig: *mut siginfo,
    // POSIX leaves behavior unspecified if this is NULL, but on both Linux and Redox, NULL is used
    // to differentiate between sigtimedwait and sigwaitinfo internally
    tp: *const timespec,
) -> c_int {
    Sys::sigtimedwait(unsafe { &*set }, unsafe { sig.as_mut() }, unsafe {
        tp.as_ref()
    })
    .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigwaitinfo.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigwaitinfo(set: *const sigset_t, sig: *mut siginfo_t) -> c_int {
    unsafe { sigtimedwait(set, sig, core::ptr::null()) }
}

pub(crate) const SIGNAL_STRINGS: [&str; 32] = [
    "Unknown signal\0",
    "Hangup\0",
    "Interrupt\0",
    "Quit\0",
    "Illegal instruction\0",
    "Trace/breakpoint trap\0",
    "Aborted\0",
    "Bus error\0",
    "Arithmetic exception\0",
    "Killed\0",
    "User defined signal 1\0",
    "Segmentation fault\0",
    "User defined signal 2\0",
    "Broken pipe\0",
    "Alarm clock\0",
    "Terminated\0",
    "Stack fault\0",
    "Child process status\0",
    "Continued\0",
    "Stopped (signal)\0",
    "Stopped\0",
    "Stopped (tty input)\0",
    "Stopped (tty output)\0",
    "Urgent I/O condition\0",
    "CPU time limit exceeded\0",
    "File size limit exceeded\0",
    "Virtual timer expired\0",
    "Profiling timer expired\0",
    "Window changed\0",
    "I/O possible\0",
    "Power failure\0",
    "Bad system call\0",
];

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/psignal.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn psignal(sig: c_int, prefix: *const c_char) {
    let c_description = usize::try_from(sig)
        .ok()
        .and_then(|idx| SIGNAL_STRINGS.get(idx))
        .unwrap_or(&SIGNAL_STRINGS[0])
        .as_ptr();
    // fprintf can affect errno, so we save errno and restore it
    let old_errno = ERRNO.get();
    // POSIX says that "prefix" shall be written if it isn't null or an empty string.
    // Otherwise, only the signal description should be written
    if prefix.is_null() {
        unsafe {
            fprintf(stderr, c"%s\n".as_ptr(), c_description);
        }
    } else {
        unsafe {
            fprintf(stderr, c"%s: %s\n".as_ptr(), prefix, c_description);
        }
    }
    ERRNO.set(old_errno);
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/psiginfo.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn psiginfo(info: *const siginfo_t, prefix: *const c_char) {
    unsafe {
        psignal((*info).si_signo, prefix);
    }
}
