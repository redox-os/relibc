//! `signal.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/signal.h.html>.

use core::{arch::global_asm, mem, ptr};

use cbitset::BitSet;

use crate::{
    error::{Errno, ResultExt},
    header::{errno, setjmp, time::timespec},
    platform::{
        self, ERRNO, Pal, PalSignal, Sys,
        types::{
            c_char, c_int, c_ulong, c_ulonglong, c_void, pid_t, pthread_attr_t, pthread_t, size_t,
            uid_t,
        },
    },
};

pub use self::sys::*;

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

pub(crate) const SIG_DFL: usize = 0;
pub(crate) const SIG_IGN: usize = 1;
pub(crate) const SIG_ERR: isize = -1;
pub(crate) const SIG_HOLD: isize = 2;

pub const SIG_BLOCK: c_int = 0;
pub const SIG_UNBLOCK: c_int = 1;
pub const SIG_SETMASK: c_int = 2;

pub const SIGEV_SIGNAL: c_int = 0;
pub const SIGEV_NONE: c_int = 1;
pub const SIGEV_THREAD: c_int = 2;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/signal.h.html>.
#[repr(C)]
#[derive(Clone, Debug)]
/// cbindgen:ignore
pub struct sigaction {
    pub sa_handler: Option<extern "C" fn(c_int)>,
    pub sa_flags: c_ulong,
    pub sa_restorer: Option<unsafe extern "C" fn()>,
    pub sa_mask: sigset_t,
}

#[repr(C)]
#[derive(Clone)]
pub struct sigaltstack {
    pub ss_sp: *mut c_void,
    pub ss_flags: c_int,
    pub ss_size: size_t,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/signal.h.html>.
#[repr(C)]
#[derive(Clone)]
pub struct sigevent {
    pub sigev_value: sigval,
    pub sigev_signo: c_int,
    pub sigev_notify: c_int,
    pub sigev_notify_function: Option<extern "C" fn(sigval)>,
    pub sigev_notify_attributes: *mut pthread_attr_t,
}

// FIXME: This struct is wrong on Linux
#[repr(C)]
#[derive(Clone, Copy)]
pub struct siginfo {
    pub si_signo: c_int,
    pub si_errno: c_int,
    pub si_code: c_int,
    pub si_pid: pid_t,
    pub si_uid: uid_t,
    pub si_addr: *mut c_void,
    pub si_status: c_int,
    pub si_value: sigval,
}

#[unsafe(no_mangle)]
pub extern "C" fn _cbindgen_export_siginfo(a: siginfo) {}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/signal.h.html>.
#[derive(Clone, Copy)]
#[repr(C)]
pub union sigval {
    pub sival_int: c_int,
    pub sival_ptr: *mut c_void,
}

/// cbindgen:ignore
pub type sigset_t = c_ulonglong;
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/signal.h.html>.
/// cbindgen:ignore
pub type siginfo_t = siginfo;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/signal.h.html>
pub type stack_t = sigaltstack;

#[cfg(target_arch = "aarch64")]
global_asm!(include_str!("sigsetjmp/aarch64/sigsetjmp.s"));

#[cfg(target_arch = "riscv64")]
global_asm!(include_str!("sigsetjmp/riscv64/sigsetjmp.s"));

#[cfg(target_arch = "x86")]
global_asm!(
    include_str!("sigsetjmp/i386/sigsetjmp.s"),
    options(att_syntax)
);

#[cfg(target_arch = "x86_64")]
global_asm!(
    include_str!("sigsetjmp/x86_64/sigsetjmp.s"),
    options(att_syntax)
);

unsafe extern "C" {
    pub fn sigsetjmp(jb: *mut u64, savemask: i32) -> i32;
}

//NOTE for the following two functions, to see why they're implemented slightly differently from their intended behavior, read
//     https://git.musl-libc.org/cgit/musl/commit/?id=583e55122e767b1586286a0d9c35e2a4027998ab
#[unsafe(no_mangle)]
unsafe extern "C" fn __sigsetjmp_tail(jb: *mut u64, ret: i32) -> i32 {
    let set = jb.wrapping_add(9);
    if ret > 0 {
        unsafe { sigprocmask(SIG_SETMASK, set, ptr::null_mut()) };
    } else {
        unsafe { sigprocmask(SIG_SETMASK, ptr::null_mut(), set) };
    }
    ret
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn siglongjmp(jb: *mut u64, ret: i32) {
    unsafe { setjmp::longjmp(jb, ret) };
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/kill.html>.
#[unsafe(no_mangle)]
pub extern "C" fn kill(pid: pid_t, sig: c_int) -> c_int {
    Sys::kill(pid, sig).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigqueue.html>.
#[unsafe(no_mangle)]
pub extern "C" fn sigqueue(pid: pid_t, sig: c_int, val: sigval) -> c_int {
    Sys::sigqueue(pid, sig, val)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/killpg.html>.
#[unsafe(no_mangle)]
pub extern "C" fn killpg(pgrp: pid_t, sig: c_int) -> c_int {
    Sys::killpg(pgrp, sig).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_kill.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_kill(thread: pthread_t, sig: c_int) -> c_int {
    let os_tid = {
        let pthread = unsafe { &*(thread as *const crate::pthread::Pthread) };
        unsafe { pthread.os_tid.get().read() }
    };
    crate::header::pthread::e(unsafe { Sys::rlct_kill(os_tid, sig as usize) })
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_sigmask.html>.
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
#[unsafe(no_mangle)]
pub extern "C" fn raise(sig: c_int) -> c_int {
    Sys::raise(sig).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigaction.html>.
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigaddset(set: *mut sigset_t, signo: c_int) -> c_int {
    if signo <= 0 || signo as usize > NSIG.max(SIGRTMAX)
    /* TODO */
    {
        platform::ERRNO.set(errno::EINVAL);
        return -1;
    }

    if let Some(set) = unsafe { (set as *mut SigSet).as_mut() } {
        set.insert(signo as usize - 1); // 0-indexed usize, please!
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigaltstack.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigaltstack(ss: *const stack_t, old_ss: *mut stack_t) -> c_int {
    unsafe {
        Sys::sigaltstack(ss.as_ref(), old_ss.as_mut())
            .map(|()| 0)
            .or_minus_one_errno()
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigdelset.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigdelset(set: *mut sigset_t, signo: c_int) -> c_int {
    if signo <= 0 || signo as usize > NSIG.max(SIGRTMAX)
    /* TODO */
    {
        platform::ERRNO.set(errno::EINVAL);
        return -1;
    }

    if let Some(set) = unsafe { (set as *mut SigSet).as_mut() } {
        set.remove(signo as usize - 1); // 0-indexed usize, please!
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigemptyset.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigemptyset(set: *mut sigset_t) -> c_int {
    if let Some(set) = unsafe { (set as *mut SigSet).as_mut() } {
        set.clear();
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigfillset.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigfillset(set: *mut sigset_t) -> c_int {
    if let Some(set) = unsafe { (set as *mut SigSet).as_mut() } {
        set.fill(.., true);
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/sighold.html>.
///
/// Present in issue 7. Removed in issue 8.
///
/// Use of this function is unspecified in a multi-threaded process.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sighold(sig: c_int) -> c_int {
    let mut pset = mem::MaybeUninit::<sigset_t>::uninit();
    unsafe { sigemptyset(pset.as_mut_ptr()) };
    let mut set = unsafe { pset.assume_init() };
    if unsafe { sigaddset(&mut set, sig) } < 0 {
        return -1;
    }
    unsafe { sigprocmask(SIG_BLOCK, &set, ptr::null_mut()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/sighold.html>.
///
/// Present in issue 7. Removed in issue 8.
///
/// Use of this function is unspecified in a multi-threaded process.
#[unsafe(no_mangle)]
pub extern "C" fn sigignore(sig: c_int) -> c_int {
    let mut psa = mem::MaybeUninit::<sigaction>::uninit();
    unsafe { sigemptyset(&mut (*psa.as_mut_ptr()).sa_mask) };
    let mut sa = unsafe { psa.assume_init() };
    sa.sa_handler = unsafe { mem::transmute(SIG_IGN) };
    sa.sa_flags = 0;
    unsafe { sigaction(sig, &mut sa, ptr::null_mut()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/siginterrupt.html>.
///
/// Marked obsolescent in issue 7. Removed in issue 8.
#[unsafe(no_mangle)]
pub extern "C" fn siginterrupt(sig: c_int, flag: c_int) -> c_int {
    let mut psa = mem::MaybeUninit::<sigaction>::uninit();
    unsafe { sigaction(sig, ptr::null_mut(), psa.as_mut_ptr()) };
    let mut sa = unsafe { psa.assume_init() };
    if flag != 0 {
        sa.sa_flags &= !SA_RESTART as c_ulong;
    } else {
        sa.sa_flags |= SA_RESTART as c_ulong;
    }

    unsafe { sigaction(sig, &mut sa, ptr::null_mut()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigismember.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigismember(set: *const sigset_t, signo: c_int) -> c_int {
    if signo <= 0 || signo as usize > NSIG.max(SIGRTMAX)
    /* TODO */
    {
        platform::ERRNO.set(errno::EINVAL);
        return -1;
    }

    if let Some(set) = unsafe { (set as *mut SigSet).as_mut() } {
        if set.contains(signo as usize - 1) {
            return 1;
        }
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/signal.html>.
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
    if unsafe { sigaction(sig, &sa, old_sa.as_mut_ptr()) } < 0 {
        mem::forget(old_sa);
        return unsafe { mem::transmute(SIG_ERR) };
    }
    unsafe { old_sa.assume_init() }.sa_handler
}

/// See <https://pubs.opengroup.org/onlinepubs/9699919799/functions/sighold.html>.
///
/// Present in issue 7. Removed in issue 8.
///
/// Use of this function is unspecified in a multi-threaded process.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sigpause(sig: c_int) -> c_int {
    let mut pset = mem::MaybeUninit::<sigset_t>::uninit();
    unsafe { sigprocmask(0, ptr::null_mut(), pset.as_mut_ptr()) };
    let mut set = unsafe { pset.assume_init() };
    if unsafe { sigdelset(&mut set, sig) } == -1 {
        return -1;
    }
    unsafe { sigsuspend(&set) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/sigpending.html>.
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
const RLCT_SIGNAL_MASK: sigset_t = (1 << ((SIGRTMIN - 1) - 1)) | (1 << (SIGRTMIN - 2) - 1);

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
    if unsafe { sigaddset(&mut set, sig) } < 0 {
        return -1;
    }
    unsafe { sigprocmask(SIG_UNBLOCK, &mut set, ptr::null_mut()) }
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
    if unsafe { sigaddset(&mut set, sig) } < 0 {
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
                || unsafe { sigprocmask(SIG_BLOCK, &mut set, &mut set) } < 0
            {
                mem::forget(old_sa);
                return sig_err;
            }
        } else {
            let mut sa = sigaction {
                sa_handler: func,
                sa_flags: 0 as c_ulong,
                sa_restorer: None, // set by platform if applicable
                sa_mask: sigset_t::default(),
            };
            unsafe { sigemptyset(&mut sa.sa_mask) };
            if unsafe { sigaction(sig, &sa, old_sa.as_mut_ptr()) } < 0
                || unsafe { sigprocmask(SIG_UNBLOCK, &mut set, &mut set) } < 0
            {
                mem::forget(old_sa);
                return sig_err;
            }
        }
    }
    if unsafe { sigismember(&mut set, sig) } == 1 {
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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn cbindgen_stupid_struct_sigevent_for_timer(_: sigevent) {}
