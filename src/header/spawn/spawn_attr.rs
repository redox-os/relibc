use core::{
    ffi::{c_int, c_short},
    mem::zeroed,
};

use bitflags;

use crate::header::{
    bits_sigset_t::sigset_t,
    errno::EINVAL,
    sched::{SCHED_FIFO, SCHED_OTHER, SCHED_RR, sched_param},
    sys_types_internal::pid_t,
};

bitflags::bitflags! {
    pub struct Flags: c_short
    {
        const POSIX_SPAWN_RESETIDS = 1;
        const POSIX_SPAWN_SETPGROUP = 2;
        const POSIX_SPAWN_SETSIGDEF = 3;
        const POSIX_SPAWN_SETSIGMASK = 4;
        const POSIX_SPAWN_SETSCHEDPARAM = 5;
        const POSIX_SPAWN_SETSCHEDULER = 6;
    }
}

pub const POSIX_SPAWN_RESETIDS: c_short = 1;
pub const POSIX_SPAWN_SETPGROUP: c_short = 2;
pub const POSIX_SPAWN_SETSIGDEF: c_short = 3;
pub const POSIX_SPAWN_SETSIGMASK: c_short = 4;
pub const POSIX_SPAWN_SETSCHEDPARAM: c_short = 5;
pub const POSIX_SPAWN_SETSCHEDULER: c_short = 6;

#[repr(C)]
pub struct posix_spawnattr_t {
    pub param: sched_param,
    pub flags: c_short,
    pub pgroup: c_int,
    policy: c_int,
    pub sigdefault: sigset_t,
    pub sigmask: sigset_t,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_spawnattr_init.html>
///
/// Panics is `attr` is `NULL`.
#[unsafe(no_mangle)]
pub extern "C" fn posix_spawnattr_init(attr: *mut posix_spawnattr_t) -> c_int {
    unsafe {
        let attr = attr.as_mut().expect("posix_spawnattr_t cannot be NULL");
        *attr = zeroed();
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_spawnattr_destroy.html>
///
/// Panics is `attr` is `NULL`.
///
/// # Safety:
/// `attr` must be a pointer to `posix_spawnattr_t` and must at least be initialised.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_destroy(attr: *mut posix_spawnattr_t) -> c_int {
    unsafe {
        let attr = attr.as_mut().expect("posix_spawnattr_t cannot be NULL");
        *attr = zeroed();
    }
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_spawnattr_setschedparam.html>
///
/// Panics if `attr` or `schedparam` is `NULL`.
///
/// # Safety:
/// `attr` must be initialised.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_setschedparam(
    attr: *mut posix_spawnattr_t,
    schedparam: *const sched_param,
) -> c_int {
    let attr = unsafe { attr.as_mut().expect("posix_spawnattr_t cannot be NULL") };
    let schedparam = unsafe { schedparam.as_ref().expect("schedparam cannot be NULL") };
    (*attr).param = *schedparam;
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_spawnattr_getschedparam.html>
///
/// Panics if `attr` or `schedparam` is `NULL`.
///
/// # Safety:
/// `attr` must be initialised.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_getschedparam(
    attr: *const posix_spawnattr_t,
    schedparam: *mut sched_param,
) -> c_int {
    let attr = unsafe { attr.as_ref().expect("posix_spawnattr_t cannot be NULL") };
    let schedparam = unsafe { schedparam.as_mut().expect("schedparam cannot be NULL") };
    *schedparam = (*attr).param;
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_spawnattr_setschedpolicy.html>
///
/// Panics if `attr` is `NULL`.
///
/// # Safety:
/// `attr` must be initialised.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_setschedpolicy(
    attr: *mut posix_spawnattr_t,
    schedpolicy: c_int,
) -> c_int {
    let attr = unsafe { attr.as_mut().expect("posix_spawnattr_t cannot be NULL") };
    match schedpolicy {
        SCHED_FIFO | SCHED_RR | SCHED_OTHER => (*attr).policy = schedpolicy,
        _ => return EINVAL,
    }

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_spawnattr_getschedpolicy.html>
///
/// Panics if `attr` or `schedpolicy` is `NULL`.
///
/// # Safety:
/// `attr` must be initialised.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_getschedpolicy(
    attr: *const posix_spawnattr_t,
    schedpolicy: *mut c_int,
) -> c_int {
    let attr = unsafe { attr.as_ref().expect("posix_spawnattr_t cannot be NULL") };
    let schedpolicy = unsafe { schedpolicy.as_mut().expect("schedpolicy cannot be NULL") };
    *schedpolicy = (*attr).policy;
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_spawnattr_setsigdefault.html>
///
/// Panics if `attr` or `sigdefault` is `NULL`.
///
/// # Safety:
/// `attr` must be initialised.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_setsigdefault(
    attr: *mut posix_spawnattr_t,
    sigdefault: *const sigset_t,
) -> c_int {
    let attr = unsafe { attr.as_mut().expect("posix_spawnattr_t cannot be NULL") };
    let sigdefault = unsafe { sigdefault.as_ref().expect("sigdefault cannot be NULL") };
    (*attr).sigdefault = *sigdefault;
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_spawnattr_getsigdefault.html>
///
/// Panics if `attr` or `sigdefault` is `NULL`.
///
/// # Safety:
/// `attr` must be initialised.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_getsigdefault(
    attr: *const posix_spawnattr_t,
    sigdefault: *mut sigset_t,
) -> c_int {
    let attr = unsafe { attr.as_ref().expect("posix_spawnattr_t cannot be NULL") };
    let sigdefault = unsafe { sigdefault.as_mut().expect("sigdefault cannot be NULL") };
    *sigdefault = (*attr).sigdefault;
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_spawnattr_setsigmask.html>
///
/// Panics if `attr` or `sigmask` is `NULL`.
///
/// # Safety:
/// `attr` must be initialised.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_setsigmask(
    attr: *mut posix_spawnattr_t,
    sigmask: *const sigset_t,
) -> c_int {
    let attr = unsafe { attr.as_mut().expect("posix_spawnattr_t cannot be NULL") };
    let sigmask = unsafe { sigmask.as_ref().expect("sigmask cannot be NULL") };
    (*attr).sigmask = *sigmask;
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_spawnattr_getsigmask.html>
///
/// Panics if `attr` or `sigmask` is `NULL`.
///
/// # Safety:
/// `attr` must be initialised.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_getsigmask(
    attr: *const posix_spawnattr_t,
    sigmask: *mut sigset_t,
) -> c_int {
    let attr = unsafe { attr.as_ref().expect("posix_spawnattr_t cannot be NULL") };
    let sigmask = unsafe { sigmask.as_mut().expect("sigmask cannot be NULL") };
    *sigmask = (*attr).sigmask;
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_spawnattr_setflags.html>
///
/// Panics if `attr` is `NULL`.
///
/// # Safety:
/// `attr` must be initialised.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_setflags(
    attr: *mut posix_spawnattr_t,
    flags: c_short,
) -> c_int {
    let attr = unsafe { attr.as_mut().expect("posix_spawnattr_t cannot be NULL") };
    match Flags::from_bits(flags) {
        Some(v) => (*attr).flags = v.bits(),
        None => {
            return EINVAL;
        }
    }

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_spawnattr_getflags.html>
///
/// Panics if `attr` or `flags` is `NULL`.
///
/// # Safety:
/// `attr` must be initialised.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_getflags(
    attr: *const posix_spawnattr_t,
    flags: *mut c_short,
) -> c_int {
    let attr = unsafe { attr.as_ref().expect("posix_spawnattr_t cannot be NULL") };
    let flags = unsafe { flags.as_mut().expect("flags cannot be NULL") };

    *flags = (*attr).flags;
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_spawnattr_setpgroup.html>
///
/// Panics if `attr` is `NULL`.
///
/// # Safety:
/// `attr` must be initialised.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_setpgroup(
    attr: *mut posix_spawnattr_t,
    pgroup: pid_t,
) -> c_int {
    let attr = unsafe { attr.as_mut().expect("posix_spawnattr_t cannot be NULL") };
    (*attr).pgroup = pgroup;
    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/posix_spawnattr_getpgroup.html>
///
/// Panics if `attr` or `pgroup` is `NULL`.
///
/// # Safety:
/// `attr` must be initialised.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_getpgroup(
    attr: *const posix_spawnattr_t,
    pgroup: *mut pid_t,
) -> c_int {
    let attr = unsafe { attr.as_ref().expect("posix_spawnattr_t cannot be NULL") };
    let pgroup = unsafe { pgroup.as_mut().expect("pgroup cannot be NULL") };
    *pgroup = (*attr).pgroup;
    0
}
