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
    struct Flags: c_short
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
    pub(crate) param: sched_param,
    flags: c_short,
    pgroup: c_int,
    policy: c_int,
    sigdefault: sigset_t,
    sigmask: sigset_t,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_init(attr: *mut posix_spawnattr_t) -> c_int {
    if attr.is_null() {
        return EINVAL;
    }

    unsafe {
        *attr = zeroed();
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_destroy(attr: *mut posix_spawnattr_t) -> c_int {
    if attr.is_null() {
        return EINVAL;
    }

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_setschedparam(
    attr: *mut posix_spawnattr_t,
    schedparam: *const sched_param,
) -> c_int {
    if attr.is_null() || schedparam.is_null() {
        return EINVAL;
    }

    unsafe {
        (*attr).param = *schedparam;
    }

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_getschedparam(
    attr: *const posix_spawnattr_t,
    schedparam: *mut sched_param,
) -> c_int {
    if attr.is_null() || schedparam.is_null() {
        return EINVAL;
    }

    unsafe {
        *schedparam = (*attr).param;
    }

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_setschedpolicy(
    attr: *mut posix_spawnattr_t,
    schedpolicy: c_int,
) -> c_int {
    if attr.is_null() {
        return EINVAL;
    }

    match schedpolicy {
        SCHED_FIFO | SCHED_RR | SCHED_OTHER => unsafe {
            (*attr).policy = schedpolicy;
        },
        _ => return EINVAL,
    }

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_getschedpolicy(
    attr: *const posix_spawnattr_t,
    schedpolicy: *mut c_int,
) -> c_int {
    if attr.is_null() || schedpolicy.is_null() {
        return EINVAL;
    }

    unsafe {
        *schedpolicy = (*attr).policy;
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_setsigdefault(
    attr: *mut posix_spawnattr_t,
    sigdefault: *const sigset_t,
) -> c_int {
    if attr.is_null() || sigdefault.is_null() {
        return EINVAL;
    }

    unsafe {
        (*attr).sigdefault = *sigdefault;
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_getsigdefault(
    attr: *const posix_spawnattr_t,
    sigdefault: *mut sigset_t,
) -> c_int {
    if attr.is_null() || sigdefault.is_null() {
        return EINVAL;
    }

    unsafe {
        *sigdefault = (*attr).sigdefault;
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_setsigmask(
    attr: *mut posix_spawnattr_t,
    sigmask: *const sigset_t,
) -> c_int {
    if attr.is_null() || sigmask.is_null() {
        return EINVAL;
    }

    unsafe {
        (*attr).sigmask = *sigmask;
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_getsigmask(
    attr: *const posix_spawnattr_t,
    sigmask: *mut sigset_t,
) -> c_int {
    if attr.is_null() || sigmask.is_null() {
        return EINVAL;
    }

    unsafe {
        *sigmask = (*attr).sigmask;
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_setflags(
    attr: *mut posix_spawnattr_t,
    flags: c_short,
) -> c_int {
    if attr.is_null() {
        return EINVAL;
    }

    unsafe {
        match Flags::from_bits(flags) {
            Some(v) => (*attr).flags = v.bits(),
            None => {
                return EINVAL;
            }
        }
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_getflags(
    attr: *const posix_spawnattr_t,
    flags: *mut c_short,
) -> c_int {
    if attr.is_null() || flags.is_null() {
        return EINVAL;
    }

    unsafe {
        *flags = (*attr).flags;
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_setpgroup(
    attr: *mut posix_spawnattr_t,
    pgroup: pid_t,
) -> c_int {
    if attr.is_null() {
        return EINVAL;
    }

    unsafe {
        (*attr).pgroup = pgroup;
    }
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_getpgroup(
    attr: *const posix_spawnattr_t,
    pgroup: *mut pid_t,
) -> c_int {
    if attr.is_null() || pgroup.is_null() {
        return EINVAL;
    }

    unsafe {
        *pgroup = (*attr).pgroup;
    }
    0
}
