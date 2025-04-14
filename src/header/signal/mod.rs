//! signal implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/signal.h.html

use core::{arch::global_asm, mem, ptr};

use cbitset::BitSet;

use crate::{
    c_str::CStr,
    error::{Errno, ResultExt},
    header::{errno, setjmp, time::timespec},
    platform::{self, types::*, Pal, PalSignal, Sys},
};

pub use self::sys::*;

use super::{errno::EFAULT, unistd};

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

pub const SI_QUEUE: c_int = -1;
pub const SI_USER: c_int = 0;

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

#[no_mangle]
pub extern "C" fn _cbindgen_export_siginfo(a: siginfo) {}

#[derive(Clone, Copy)]
#[repr(C)]
pub union sigval {
    pub sival_int: c_int,
    pub sival_ptr: *mut c_void,
}

/// cbindgen:ignore
pub type sigset_t = c_ulonglong;
/// cbindgen:ignore
pub type siginfo_t = siginfo;

pub type stack_t = sigaltstack;

#[cfg(target_arch = "aarch64")]
global_asm!(include_str!("sigsetjmp/aarch64/sigsetjmp.s"));

#[cfg(target_arch = "riscv64gc")]
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

extern "C" {
    pub fn sigsetjmp(jb: *mut u64, savemask: i32) -> i32;
}

//NOTE for the following two functions, to see why they're implemented slightly differently from their intended behavior, read
//     https://git.musl-libc.org/cgit/musl/commit/?id=583e55122e767b1586286a0d9c35e2a4027998ab
#[no_mangle]
unsafe extern "C" fn __sigsetjmp_tail(jb: *mut u64, ret: i32) -> i32 {
    let set = jb.wrapping_add(9);
    if ret > 0 {
        sigprocmask(SIG_SETMASK, set, ptr::null_mut());
    } else {
        sigprocmask(SIG_SETMASK, ptr::null_mut(), set);
    }
    ret
}

#[no_mangle]
pub unsafe extern "C" fn siglongjmp(jb: *mut u64, ret: i32) {
    setjmp::longjmp(jb, ret);
}

#[no_mangle]
pub extern "C" fn kill(pid: pid_t, sig: c_int) -> c_int {
    Sys::kill(pid, sig).map(|()| 0).or_minus_one_errno()
}
#[no_mangle]
pub extern "C" fn sigqueue(pid: pid_t, sig: c_int, val: sigval) -> c_int {
    Sys::sigqueue(pid, sig, val)
        .map(|()| 0)
        .or_minus_one_errno()
}

#[no_mangle]
pub extern "C" fn killpg(pgrp: pid_t, sig: c_int) -> c_int {
    Sys::killpg(pgrp, sig).map(|()| 0).or_minus_one_errno()
}

#[no_mangle]
pub unsafe extern "C" fn pthread_kill(thread: pthread_t, sig: c_int) -> c_int {
    let os_tid = {
        let pthread = &*(thread as *const crate::pthread::Pthread);
        pthread.os_tid.get().read()
    };
    crate::header::pthread::e(Sys::rlct_kill(os_tid, sig as usize))
}

#[no_mangle]
pub unsafe extern "C" fn pthread_sigmask(
    how: c_int,
    set: *const sigset_t,
    oldset: *mut sigset_t,
) -> c_int {
    // On Linux and Redox, pthread_sigmask and sigprocmask are equivalent
    if sigprocmask(how, set, oldset) == 0 {
        0
    } else {
        //TODO: Fix race
        platform::ERRNO.get()
    }
}

#[no_mangle]
pub extern "C" fn raise(sig: c_int) -> c_int {
    Sys::raise(sig).map(|()| 0).or_minus_one_errno()
}

#[no_mangle]
pub unsafe extern "C" fn sigaction(
    sig: c_int,
    act: *const sigaction,
    oact: *mut sigaction,
) -> c_int {
    Sys::sigaction(sig, act.as_ref(), oact.as_mut())
        .map(|()| 0)
        .or_minus_one_errno()
}

#[no_mangle]
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

#[no_mangle]
pub unsafe extern "C" fn sigaltstack(ss: *const stack_t, old_ss: *mut stack_t) -> c_int {
    Sys::sigaltstack(ss.as_ref(), old_ss.as_mut())
        .map(|()| 0)
        .or_minus_one_errno()
}

#[no_mangle]
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

#[no_mangle]
pub unsafe extern "C" fn sigemptyset(set: *mut sigset_t) -> c_int {
    if let Some(set) = (set as *mut SigSet).as_mut() {
        set.clear();
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigfillset(set: *mut sigset_t) -> c_int {
    if let Some(set) = (set as *mut SigSet).as_mut() {
        set.fill(.., true);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn sighold(sig: c_int) -> c_int {
    let mut pset = mem::MaybeUninit::<sigset_t>::uninit();
    unsafe { sigemptyset(pset.as_mut_ptr()) };
    let mut set = unsafe { pset.assume_init() };
    if sigaddset(&mut set, sig) < 0 {
        return -1;
    }
    sigprocmask(SIG_BLOCK, &set, ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn sigignore(sig: c_int) -> c_int {
    let mut psa = mem::MaybeUninit::<sigaction>::uninit();
    unsafe { sigemptyset(&mut (*psa.as_mut_ptr()).sa_mask) };
    let mut sa = unsafe { psa.assume_init() };
    sa.sa_handler = unsafe { mem::transmute(SIG_IGN) };
    sa.sa_flags = 0;
    unsafe { sigaction(sig, &mut sa, ptr::null_mut()) }
}

#[no_mangle]
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

#[no_mangle]
pub unsafe extern "C" fn sigismember(set: *const sigset_t, signo: c_int) -> c_int {
    if signo <= 0 || signo as usize > NSIG {
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

#[no_mangle]
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

#[no_mangle]
pub unsafe extern "C" fn sigpause(sig: c_int) -> c_int {
    let mut pset = mem::MaybeUninit::<sigset_t>::uninit();
    sigprocmask(0, ptr::null_mut(), pset.as_mut_ptr());
    let mut set = pset.assume_init();
    sigdelset(&mut set, sig);
    sigsuspend(&set)
}

#[no_mangle]
pub unsafe extern "C" fn sigpending(set: *mut sigset_t) -> c_int {
    (|| Sys::sigpending(set.as_mut().ok_or(Errno(EFAULT))?))()
        .map(|()| 0)
        .or_minus_one_errno()
}

const BELOW_SIGRTMIN_MASK: sigset_t = (1 << SIGRTMIN) - 1;
const STANDARD_SIG_MASK: sigset_t = (1 << 32) - 1;
const RLCT_SIGNAL_MASK: sigset_t = BELOW_SIGRTMIN_MASK & !STANDARD_SIG_MASK;

#[no_mangle]
pub unsafe extern "C" fn sigprocmask(
    how: c_int,
    set: *const sigset_t,
    oset: *mut sigset_t,
) -> c_int {
    (|| {
        let set = set.as_ref().map(|&block| block & !RLCT_SIGNAL_MASK);
        let mut oset = oset.as_mut();

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

#[no_mangle]
pub unsafe extern "C" fn sigrelse(sig: c_int) -> c_int {
    let mut pset = mem::MaybeUninit::<sigset_t>::uninit();
    sigemptyset(pset.as_mut_ptr());
    let mut set = pset.assume_init();
    if sigaddset(&mut set, sig) < 0 {
        return -1;
    }
    sigprocmask(SIG_UNBLOCK, &mut set, ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn sigset(
    sig: c_int,
    func: Option<extern "C" fn(c_int)>,
) -> Option<extern "C" fn(c_int)> {
    let mut old_sa = mem::MaybeUninit::uninit();
    let mut pset = mem::MaybeUninit::<sigset_t>::uninit();
    let sig_hold: Option<extern "C" fn(c_int)> = mem::transmute(SIG_HOLD);
    let sig_err: Option<extern "C" fn(c_int)> = mem::transmute(SIG_ERR);
    sigemptyset(pset.as_mut_ptr());
    let mut set = pset.assume_init();
    if sigaddset(&mut set, sig) < 0 {
        return sig_err;
    } else {
        if func == sig_hold {
            if sigaction(sig, ptr::null_mut(), old_sa.as_mut_ptr()) < 0
                || sigprocmask(SIG_BLOCK, &mut set, &mut set) < 0
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
            sigemptyset(&mut sa.sa_mask);
            if sigaction(sig, &sa, old_sa.as_mut_ptr()) < 0
                || sigprocmask(SIG_UNBLOCK, &mut set, &mut set) < 0
            {
                mem::forget(old_sa);
                return sig_err;
            }
        }
    }
    if sigismember(&mut set, sig) == 1 {
        return sig_hold;
    }
    old_sa.assume_init().sa_handler
}

#[no_mangle]
pub unsafe extern "C" fn sigsuspend(sigmask: *const sigset_t) -> c_int {
    Err(Sys::sigsuspend(&*sigmask)).or_minus_one_errno()
}

#[no_mangle]
pub unsafe extern "C" fn sigwait(set: *const sigset_t, sig: *mut c_int) -> c_int {
    let mut pinfo = mem::MaybeUninit::<siginfo_t>::uninit();
    if sigtimedwait(set, pinfo.as_mut_ptr(), ptr::null_mut()) < 0 {
        return -1;
    }
    let info = pinfo.assume_init();
    (*sig) = info.si_signo;
    0
}

#[no_mangle]
pub unsafe extern "C" fn sigtimedwait(
    set: *const sigset_t,
    // s/siginfo_t/siginfo due to https://github.com/mozilla/cbindgen/issues/621
    sig: *mut siginfo,
    // POSIX leaves behavior unspecified if this is NULL, but on both Linux and Redox, NULL is used
    // to differentiate between sigtimedwait and sigwaitinfo internally
    tp: *const timespec,
) -> c_int {
    Sys::sigtimedwait(&*set, sig.as_mut(), tp.as_ref())
        .map(|()| 0)
        .or_minus_one_errno()
}
#[no_mangle]
pub unsafe extern "C" fn sigwaitinfo(set: *const sigset_t, sig: *mut siginfo_t) -> c_int {
    sigtimedwait(set, sig, core::ptr::null())
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
#[no_mangle]
pub unsafe extern "C" fn psignal(sig: c_int, prefix: *const c_char) {
    let c_description = usize::try_from(sig)
        .ok()
        .and_then(|idx| SIGNAL_STRINGS.get(idx))
        .unwrap_or(&SIGNAL_STRINGS[0]);
    let description = &c_description[..c_description.len() - 1];
    let prefix = CStr::from_ptr(prefix).to_string_lossy();
    // TODO: stack vec or print directly?
    let string = alloc::format!("{prefix}:{description}\n");
    // TODO: better internal libc API?
    let _ = unistd::write(
        unistd::STDERR_FILENO,
        string.as_bytes().as_ptr().cast(),
        string.as_bytes().len(),
    );
}
#[no_mangle]
pub unsafe extern "C" fn psiginfo(info: *const siginfo_t, prefix: *const c_char) {
    let siginfo_t {
        si_code,
        si_signo,
        si_pid,
        si_uid,
        si_errno,
        si_addr,
        si_status,
        si_value,
    } = &*info;
    let sival_ptr = si_value.sival_ptr;
    let prefix = CStr::from_ptr(prefix).to_string_lossy();
    // TODO: stack vec or print directly?
    let string = alloc::format!(
        "{prefix}:siginfo_t {{
    si_code: {si_code}
    si_signo: {si_signo}
    si_pid: {si_pid}
    si_uid: {si_uid}
    si_errno: {si_errno}
    si_addr: {si_addr:p}
    si_status: {si_status}
    si_value: {sival_ptr:p}
}}
"
    );
    // TODO: better internal libc API?
    let _ = unistd::write(
        unistd::STDERR_FILENO,
        string.as_bytes().as_ptr().cast(),
        string.as_bytes().len(),
    );
}
