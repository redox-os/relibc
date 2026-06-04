//! `sys/resource.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_resource.h.html>.

use crate::{
    error::ResultExt,
    header::sys_select::timeval,
    out::Out,
    platform::{
        Pal, Sys,
        types::{c_int, c_long, c_ulonglong, id_t},
    },
};

/// Returns information about the current process.
pub const RUSAGE_SELF: c_int = 0;
/// Returns information about children of the current process.
pub const RUSAGE_CHILDREN: c_int = -1;
/// Non-POSIX.
///
/// Return resource consumption statistics for both the current process and
/// all of its terminated child processes that have been waited for.
pub const RUSAGE_BOTH: c_int = -2;
// TODO should be guarded by `_GNU_SOURCE`
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrusage.2.html>.
///
/// Return resource usage statistics for the calling thread.
pub const RUSAGE_THREAD: c_int = 1;

/// A value of `rlim_t` indicating no limit.
pub const RLIM_INFINITY: u64 = 0xFFFF_FFFF_FFFF_FFFF;
/// A value of type `rlim_t` indicating an unrepresentable saved soft limit.
pub const RLIM_SAVED_CUR: u64 = RLIM_INFINITY;
/// A value of type `rlim_t` indicating an unrepresentable saved hard limit.
pub const RLIM_SAVED_MAX: u64 = RLIM_INFINITY;

/// Limit on CPU time per process.
pub const RLIMIT_CPU: c_int = 0;
/// Limit on file size.
pub const RLIMIT_FSIZE: c_int = 1;
/// Limit on data segment size.
pub const RLIMIT_DATA: c_int = 2;
/// Limit on stack size.
pub const RLIMIT_STACK: c_int = 3;
/// Limit on size of core image.
pub const RLIMIT_CORE: c_int = 4;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrlimit.2.html>.
///
/// This is a limit (in bytes) on the process's resident set (the number of
/// virtual pages resident in RAM).
/// Only affects Linux 2.4.0 to 2.4.29.
pub const RLIMIT_RSS: c_int = 5;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrlimit.2.html>.
///
/// This is a limit on the number of extant processes (or, more preciselu on
/// Linux, threads) for the real user ID of the calling process.
pub const RLIMIT_NPROC: c_int = 6;
/// Limit on number of open files.
pub const RLIMIT_NOFILE: c_int = 7;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrlimit.2.html>.
///
/// This is the maximum number of bytes of memory that may be locked into RAM.
pub const RLIMIT_MEMLOCK: c_int = 8;
/// Limit on address space size.
pub const RLIMIT_AS: c_int = 9;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrlimit.2.html>.
///
/// This is a limit on the combined number of `flock(2)` locks and `fcntl(2)`
/// leases that this process may establish.
/// Only affects Linux 2.4.0 to 2.4.24.
pub const RLIMIT_LOCKS: c_int = 10;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrlimit.2.html>.
///
/// This is a limit on the number of signals that may be queued for the real
/// user ID of the calling process.
pub const RLIMIT_SIGPENDING: c_int = 11;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrlimit.2.html>.
///
/// This is a limit on the number of bytes that can be allocated for POSIX
/// message queues for the real user ID of the calling process.
pub const RLIMIT_MSGQUEUE: c_int = 12;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrlimit.2.html>.
///
/// This specifies a ceiling to which the process's nice value can be raised
/// using `setpriority(2)` or `nice(2)`.
pub const RLIMIT_NICE: c_int = 13;
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrlimit.2.html>.
///
/// This specifies a ceiling on the real-time priority that may be set for
/// this process using `sched_setscheduler(2)` and `sched_setparam(2)`.
pub const RLIMIT_RTPRIO: c_int = 14;
/// Non-POSIX, found in glibc.
///
/// Number of limit flavors.
pub const RLIMIT_NLIMITS: c_int = 15;

/// Unsigned integer type used for limit values.
pub type rlim_t = c_ulonglong;

#[repr(C)]
pub struct rlimit {
    /// The current (soft) limit.
    pub rlim_cur: rlim_t,
    /// The hard limit.
    pub rlim_max: rlim_t,
}

#[repr(C)]
pub struct rusage {
    /// User time used.
    pub ru_utime: timeval,
    /// System time used.
    pub ru_stime: timeval,
    /// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrusage.2.html>.
    ///
    /// Maximum resident set size.
    pub ru_maxrss: c_long,
    /// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrusage.2.html>.
    ///
    /// Integral shared memory size.
    pub ru_ixrss: c_long,
    /// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrusage.2.html>.
    ///
    /// Integral unshared data size.
    pub ru_idrss: c_long,
    /// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrusage.2.html>.
    ///
    /// Integral unshared stack size.
    pub ru_isrss: c_long,
    /// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrusage.2.html>.
    ///
    /// Page reclaims (soft page faults).
    pub ru_minflt: c_long,
    /// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrusage.2.html>.
    ///
    /// Page faults (hard page faults).
    pub ru_majflt: c_long,
    /// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrusage.2.html>.
    ///
    /// Swaps.
    pub ru_nswap: c_long,
    /// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrusage.2.html>.
    ///
    /// Block input operations.
    pub ru_inblock: c_long,
    /// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrusage.2.html>.
    ///
    /// Block output operations.
    pub ru_oublock: c_long,
    /// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrusage.2.html>.
    ///
    /// IPC messages sent.
    pub ru_msgsnd: c_long,
    /// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrusage.2.html>.
    ///
    /// IPC messages received.
    pub ru_msgrcv: c_long,
    /// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrusage.2.html>.
    ///
    /// Signals received.
    pub ru_nsignals: c_long,
    /// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrusage.2.html>.
    ///
    /// Voluntary context switches.
    pub ru_nvcsw: c_long,
    /// Non-POSIX, see <https://www.man7.org/linux/man-pages/man2/getrusage.2.html>.
    ///
    /// Involuntary context switches.
    pub ru_nivcsw: c_long,
}

/// Identifies the `who` argument as a process ID.
pub const PRIO_PROCESS: c_int = 0;
/// Identifies the `who` argument as a process group ID.
pub const PRIO_PGRP: c_int = 1;
/// Identifies the `who` argument as a user ID.
pub const PRIO_USER: c_int = 2;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getpriority.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getpriority(which: c_int, who: id_t) -> c_int {
    let r = Sys::getpriority(which, who).or_minus_one_errno();
    if r < 0 {
        return r;
    }
    20 - r
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/setpriority.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setpriority(which: c_int, who: id_t, nice: c_int) -> c_int {
    Sys::setpriority(which, who, nice)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getrlimit.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getrlimit(resource: c_int, rlp: *mut rlimit) -> c_int {
    let rlp = unsafe { Out::nonnull(rlp) };

    Sys::getrlimit(resource, rlp)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/setrlimit.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setrlimit(resource: c_int, rlp: *const rlimit) -> c_int {
    unsafe { Sys::setrlimit(resource, rlp) }
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getrusage.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getrusage(who: c_int, r_usage: *mut rusage) -> c_int {
    Sys::getrusage(who, unsafe { Out::nonnull(r_usage) })
        .map(|()| 0)
        .or_minus_one_errno()
}
