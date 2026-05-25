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
    pub(crate) param: sched_param,
    pub(crate) flags: c_short,
    pub(crate) pgroup: c_int,
    policy: c_int,
    sigdefault: sigset_t,
    pub(crate) sigmask: sigset_t,
}

#[unsafe(no_mangle)]
pub extern "C" fn posix_spawnattr_init(attr: &mut posix_spawnattr_t) -> c_int {
    *attr = unsafe { zeroed() };
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn posix_spawnattr_destroy(attr: &mut posix_spawnattr_t) -> c_int {
    *attr = unsafe { zeroed() };
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_setschedparam(
    attr: &mut posix_spawnattr_t,
    schedparam: &sched_param,
) -> c_int {
    (*attr).param = *schedparam;
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_getschedparam(
    attr: &posix_spawnattr_t,
    schedparam: &mut sched_param,
) -> c_int {
    *schedparam = (*attr).param;
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_setschedpolicy(
    attr: &mut posix_spawnattr_t,
    schedpolicy: c_int,
) -> c_int {
    match schedpolicy {
        SCHED_FIFO | SCHED_RR | SCHED_OTHER => (*attr).policy = schedpolicy,
        _ => return EINVAL,
    }

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_getschedpolicy(
    attr: &posix_spawnattr_t,
    schedpolicy: &mut c_int,
) -> c_int {
    *schedpolicy = (*attr).policy;
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_setsigdefault(
    attr: &mut posix_spawnattr_t,
    sigdefault: &sigset_t,
) -> c_int {
    (*attr).sigdefault = *sigdefault;
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_getsigdefault(
    attr: &posix_spawnattr_t,
    sigdefault: &mut sigset_t,
) -> c_int {
    *sigdefault = (*attr).sigdefault;
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_setsigmask(
    attr: &mut posix_spawnattr_t,
    sigmask: &sigset_t,
) -> c_int {
    (*attr).sigmask = *sigmask;
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_getsigmask(
    attr: &posix_spawnattr_t,
    sigmask: &mut sigset_t,
) -> c_int {
    *sigmask = (*attr).sigmask;
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_setflags(
    attr: &mut posix_spawnattr_t,
    flags: c_short,
) -> c_int {
    match Flags::from_bits(flags) {
        Some(v) => (*attr).flags = v.bits(),
        None => {
            return EINVAL;
        }
    }

    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_getflags(
    attr: &posix_spawnattr_t,
    flags: &mut c_short,
) -> c_int {
    *flags = (*attr).flags;
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_setpgroup(
    attr: &mut posix_spawnattr_t,
    pgroup: pid_t,
) -> c_int {
    (*attr).pgroup = pgroup;
    0
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn posix_spawnattr_getpgroup(
    attr: &posix_spawnattr_t,
    pgroup: &mut pid_t,
) -> c_int {
    *pgroup = (*attr).pgroup;
    0
}
