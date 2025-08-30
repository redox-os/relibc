/// Linux sysconf implementation.
/// Constants borrowed from musl.
use core::{
    ffi::{c_int, c_long},
    mem::size_of,
};

use crate::{
    header::{
        errno,
        limits::{HOST_NAME_MAX, PAGE_SIZE},
        signal,
    },
    platform,
};

pub const _POSIX2_BC_BASE_MAX: c_long = 99;
pub const _POSIX2_BC_DIM_MAX: c_long = 2048;
pub const _POSIX2_BC_SCALE_MAX: c_long = 99;
pub const _POSIX2_BC_STRING_MAX: c_long = 1000;
pub const _POSIX2_CHARCLASS_NAME_MAX: c_long = 14;
pub const _POSIX2_COLL_WEIGHTS_MAX: c_long = 2;
pub const _POSIX2_EXPR_NEST_MAX: c_long = 32;
pub const _POSIX2_LINE_MAX: c_long = 2048;
pub const _POSIX2_RE_DUP_MAX: c_long = 255;
pub const _XOPEN_IOV_MAX: c_long = 16;
pub const _XOPEN_NAME_MAX: c_long = 255;
pub const _XOPEN_PATH_MAX: c_long = 1024;

pub const _SC_ARG_MAX: c_int = 0;
pub const _SC_CHILD_MAX: c_int = 1;
pub const _SC_CLK_TCK: c_int = 2;
pub const _SC_NGROUPS_MAX: c_int = 3;
pub const _SC_OPEN_MAX: c_int = 4;
pub const _SC_STREAM_MAX: c_int = 5;
pub const _SC_TZNAME_MAX: c_int = 6;
pub const _SC_JOB_CONTROL: c_int = 7;
pub const _SC_SAVED_IDS: c_int = 8;
pub const _SC_REALTIME_SIGNALS: c_int = 9;
pub const _SC_PRIORITY_SCHEDULING: c_int = 10;
pub const _SC_TIMERS: c_int = 11;
pub const _SC_ASYNCHRONOUS_IO: c_int = 12;
pub const _SC_PRIORITIZED_IO: c_int = 13;
pub const _SC_SYNCHRONIZED_IO: c_int = 14;
pub const _SC_FSYNC: c_int = 15;
pub const _SC_MAPPED_FILES: c_int = 16;
pub const _SC_MEMLOCK: c_int = 17;
pub const _SC_MEMLOCK_RANGE: c_int = 18;
pub const _SC_MEMORY_PROTECTION: c_int = 19;
pub const _SC_MESSAGE_PASSING: c_int = 20;
pub const _SC_SEMAPHORES: c_int = 21;
pub const _SC_SHARED_MEMORY_OBJECTS: c_int = 22;
pub const _SC_AIO_LISTIO_MAX: c_int = 23;
pub const _SC_AIO_MAX: c_int = 24;
pub const _SC_AIO_PRIO_DELTA_MAX: c_int = 25;
pub const _SC_DELAYTIMER_MAX: c_int = 26;
pub const _SC_MQ_OPEN_MAX: c_int = 27;
pub const _SC_MQ_PRIO_MAX: c_int = 28;
pub const _SC_VERSION: c_int = 29;
pub const _SC_PAGE_SIZE: c_int = 30;
pub const _SC_PAGESIZE: c_int = 30;
pub const _SC_RTSIG_MAX: c_int = 31;
pub const _SC_SEM_NSEMS_MAX: c_int = 32;
pub const _SC_SEM_VALUE_MAX: c_int = 33;
pub const _SC_SIGQUEUE_MAX: c_int = 34;
pub const _SC_TIMER_MAX: c_int = 35;
pub const _SC_BC_BASE_MAX: c_int = 36;
pub const _SC_BC_DIM_MAX: c_int = 37;
pub const _SC_BC_SCALE_MAX: c_int = 38;
pub const _SC_BC_STRING_MAX: c_int = 39;
pub const _SC_COLL_WEIGHTS_MAX: c_int = 40;
pub const _SC_EXPR_NEST_MAX: c_int = 42;
pub const _SC_LINE_MAX: c_int = 43;
pub const _SC_RE_DUP_MAX: c_int = 44;
pub const _SC_2_VERSION: c_int = 46;
pub const _SC_2_C_BIND: c_int = 47;
pub const _SC_2_C_DEV: c_int = 48;
pub const _SC_2_FORT_DEV: c_int = 49;
pub const _SC_2_FORT_RUN: c_int = 50;
pub const _SC_2_SW_DEV: c_int = 51;
pub const _SC_2_LOCALEDEF: c_int = 52;
pub const _SC_UIO_MAXIOV: c_int = 60;
pub const _SC_IOV_MAX: c_int = 60;
pub const _SC_THREADS: c_int = 67;
pub const _SC_THREAD_SAFE_FUNCTIONS: c_int = 68;
pub const _SC_GETGR_R_SIZE_MAX: c_int = 69;
pub const _SC_GETPW_R_SIZE_MAX: c_int = 70;
pub const _SC_LOGIN_NAME_MAX: c_int = 71;
pub const _SC_TTY_NAME_MAX: c_int = 72;
pub const _SC_THREAD_DESTRUCTOR_ITERATIONS: c_int = 73;
pub const _SC_THREAD_KEYS_MAX: c_int = 74;
pub const _SC_THREAD_STACK_MIN: c_int = 75;
pub const _SC_THREAD_THREADS_MAX: c_int = 76;
pub const _SC_THREAD_ATTR_STACKADDR: c_int = 77;
pub const _SC_THREAD_ATTR_STACKSIZE: c_int = 78;
pub const _SC_THREAD_PRIORITY_SCHEDULING: c_int = 79;
pub const _SC_THREAD_PRIO_INHERIT: c_int = 80;
pub const _SC_THREAD_PRIO_PROTECT: c_int = 81;
pub const _SC_THREAD_PROCESS_SHARED: c_int = 82;
pub const _SC_NPROCESSORS_CONF: c_int = 83;
pub const _SC_NPROCESSORS_ONLN: c_int = 84;
pub const _SC_PHYS_PAGES: c_int = 85;
pub const _SC_AVPHYS_PAGES: c_int = 86;
pub const _SC_ATEXIT_MAX: c_int = 87;
pub const _SC_PASS_MAX: c_int = 88;
pub const _SC_XOPEN_VERSION: c_int = 89;
pub const _SC_XOPEN_XCU_VERSION: c_int = 90;
pub const _SC_XOPEN_UNIX: c_int = 91;
pub const _SC_XOPEN_CRYPT: c_int = 92;
pub const _SC_XOPEN_ENH_I18N: c_int = 93;
pub const _SC_XOPEN_SHM: c_int = 94;
pub const _SC_2_CHAR_TERM: c_int = 95;
pub const _SC_2_UPE: c_int = 97;
pub const _SC_XOPEN_XPG2: c_int = 98;
pub const _SC_XOPEN_XPG3: c_int = 99;
pub const _SC_XOPEN_XPG4: c_int = 100;
pub const _SC_NZERO: c_int = 109;
pub const _SC_XBS5_ILP32_OFF32: c_int = 125;
pub const _SC_XBS5_ILP32_OFFBIG: c_int = 126;
pub const _SC_XBS5_LP64_OFF64: c_int = 127;
pub const _SC_XBS5_LPBIG_OFFBIG: c_int = 128;
pub const _SC_XOPEN_LEGACY: c_int = 129;
pub const _SC_XOPEN_REALTIME: c_int = 130;
pub const _SC_XOPEN_REALTIME_THREADS: c_int = 131;
pub const _SC_ADVISORY_INFO: c_int = 132;
pub const _SC_BARRIERS: c_int = 133;
pub const _SC_CLOCK_SELECTION: c_int = 137;
pub const _SC_CPUTIME: c_int = 138;
pub const _SC_THREAD_CPUTIME: c_int = 139;
pub const _SC_MONOTONIC_CLOCK: c_int = 149;
pub const _SC_READER_WRITER_LOCKS: c_int = 153;
pub const _SC_SPIN_LOCKS: c_int = 154;
pub const _SC_REGEXP: c_int = 155;
pub const _SC_SHELL: c_int = 157;
pub const _SC_SPAWN: c_int = 159;
pub const _SC_SPORADIC_SERVER: c_int = 160;
pub const _SC_THREAD_SPORADIC_SERVER: c_int = 161;
pub const _SC_TIMEOUTS: c_int = 164;
pub const _SC_TYPED_MEMORY_OBJECTS: c_int = 165;
pub const _SC_2_PBS: c_int = 168;
pub const _SC_2_PBS_ACCOUNTING: c_int = 169;
pub const _SC_2_PBS_LOCATE: c_int = 170;
pub const _SC_2_PBS_MESSAGE: c_int = 171;
pub const _SC_2_PBS_TRACK: c_int = 172;
pub const _SC_SYMLOOP_MAX: c_int = 173;
pub const _SC_STREAMS: c_int = 174;
pub const _SC_2_PBS_CHECKPOINT: c_int = 175;
pub const _SC_V6_ILP32_OFF32: c_int = 176;
pub const _SC_V6_ILP32_OFFBIG: c_int = 177;
pub const _SC_V6_LP64_OFF64: c_int = 178;
pub const _SC_V6_LPBIG_OFFBIG: c_int = 179;
pub const _SC_HOST_NAME_MAX: c_int = 180;
pub const _SC_TRACE: c_int = 181;
pub const _SC_TRACE_EVENT_FILTER: c_int = 182;
pub const _SC_TRACE_INHERIT: c_int = 183;
pub const _SC_TRACE_LOG: c_int = 184;

pub const _SC_IPV6: c_int = 235;
pub const _SC_RAW_SOCKETS: c_int = 236;
pub const _SC_V7_ILP32_OFF32: c_int = 237;
pub const _SC_V7_ILP32_OFFBIG: c_int = 238;
pub const _SC_V7_LP64_OFF64: c_int = 239;
pub const _SC_V7_LPBIG_OFFBIG: c_int = 240;
pub const _SC_SS_REPL_MAX: c_int = 241;
pub const _SC_TRACE_EVENT_NAME_MAX: c_int = 242;
pub const _SC_TRACE_NAME_MAX: c_int = 243;
pub const _SC_TRACE_SYS_MAX: c_int = 244;
pub const _SC_TRACE_USER_EVENT_MAX: c_int = 245;
pub const _SC_XOPEN_STREAMS: c_int = 246;
pub const _SC_THREAD_ROBUST_PRIO_INHERIT: c_int = 247;
pub const _SC_THREAD_ROBUST_PRIO_PROTECT: c_int = 248;
pub const _SC_MINSIGSTKSZ: c_int = 249;
pub const _SC_SIGSTKSZ: c_int = 250;

// Defined in unistd.h but we defined it in C
const _POSIX_VERSION: c_long = 200809;
const _XOPEN_VERSION: c_long = 700;

pub(super) fn sysconf_impl(name: c_int) -> c_long {
    // Values from musl which we can assume is correct.
    match name {
        _SC_CLK_TCK => 100,
        // TODO: getrlimit
        _SC_CHILD_MAX => -1,
        _SC_NGROUPS_MAX => 32,
        // TODO: getrlimit
        _SC_OPEN_MAX => -1,
        _SC_STREAM_MAX => -1,
        // TODO: limits.h
        _SC_TZNAME_MAX => -1,
        _SC_JOB_CONTROL => 1,
        _SC_SAVED_IDS => 1,
        _SC_REALTIME_SIGNALS => _POSIX_VERSION,
        _SC_PRIORITY_SCHEDULING => -1,
        _SC_TIMERS => _POSIX_VERSION,
        _SC_ASYNCHRONOUS_IO => _POSIX_VERSION,
        _SC_PRIORITIZED_IO => -1,
        _SC_SYNCHRONIZED_IO => -1,
        _SC_FSYNC => _POSIX_VERSION,
        _SC_MAPPED_FILES => _POSIX_VERSION,
        _SC_MEMLOCK => _POSIX_VERSION,
        _SC_MEMLOCK_RANGE => _POSIX_VERSION,
        _SC_MEMORY_PROTECTION => _POSIX_VERSION,
        _SC_MESSAGE_PASSING => _POSIX_VERSION,
        _SC_SEMAPHORES => _POSIX_VERSION,
        _SC_SHARED_MEMORY_OBJECTS => _POSIX_VERSION,
        _SC_AIO_LISTIO_MAX => -1,
        _SC_AIO_MAX => -1,
        _SC_AIO_PRIO_DELTA_MAX => 0,
        // TODO: limits.h?
        _SC_DELAYTIMER_MAX => -1,
        _SC_MQ_OPEN_MAX => -1,
        // TODO: limits.h?
        _SC_MQ_PRIO_MAX => -1,
        _SC_VERSION => _POSIX_VERSION,
        _SC_PAGE_SIZE => PAGE_SIZE.try_into().unwrap_or(-1),
        _SC_RTSIG_MAX => (signal::SIGRTMAX - signal::SIGRTMIN)
            .try_into()
            .unwrap_or(-1),
        // TODO: limits.h
        _SC_SEM_NSEMS_MAX => -1,
        // TODO: limits.h
        _SC_SEM_VALUE_MAX => -1,
        _SC_SIGQUEUE_MAX => -1,
        _SC_TIMER_MAX => -1,
        _SC_BC_BASE_MAX => _POSIX2_BC_BASE_MAX,
        _SC_BC_DIM_MAX => _POSIX2_BC_DIM_MAX,
        _SC_BC_SCALE_MAX => _POSIX2_BC_SCALE_MAX,
        _SC_BC_STRING_MAX => _POSIX2_BC_STRING_MAX,
        _SC_COLL_WEIGHTS_MAX => _POSIX2_COLL_WEIGHTS_MAX,
        _SC_EXPR_NEST_MAX => -1,
        _SC_LINE_MAX => -1,
        _SC_RE_DUP_MAX => _POSIX2_RE_DUP_MAX,
        _SC_2_VERSION => _POSIX_VERSION,
        _SC_2_C_BIND => _POSIX_VERSION,
        _SC_2_C_DEV => -1,
        _SC_2_FORT_DEV => -1,
        _SC_2_FORT_RUN => -1,
        _SC_2_SW_DEV => -1,
        _SC_2_LOCALEDEF => -1,
        _SC_IOV_MAX => _XOPEN_IOV_MAX,
        _SC_THREADS => _POSIX_VERSION,
        _SC_THREAD_SAFE_FUNCTIONS => _POSIX_VERSION,
        _SC_GETGR_R_SIZE_MAX => -1,
        _SC_GETPW_R_SIZE_MAX => -1,
        _SC_LOGIN_NAME_MAX => 256,
        // TODO: limits.h
        _SC_TTY_NAME_MAX => -1,
        // TODO: limits.h
        _SC_THREAD_DESTRUCTOR_ITERATIONS => -1,
        // TODO: limits.h
        _SC_THREAD_KEYS_MAX => -1,
        // TODO: limits.h
        _SC_THREAD_STACK_MIN => -1,
        _SC_THREAD_THREADS_MAX => -1,
        _SC_THREAD_ATTR_STACKADDR => _POSIX_VERSION,
        _SC_THREAD_ATTR_STACKSIZE => _POSIX_VERSION,
        _SC_THREAD_PRIORITY_SCHEDULING => _POSIX_VERSION,
        _SC_THREAD_PRIO_INHERIT => -1,
        _SC_THREAD_PRIO_PROTECT => -1,
        _SC_THREAD_PROCESS_SHARED => _POSIX_VERSION,
        // TODO: Use getaffinity syscall on Linux
        _SC_NPROCESSORS_CONF => -1,
        _SC_NPROCESSORS_ONLN => -1,
        // TODO: sysinfo
        _SC_PHYS_PAGES => -1,
        _SC_AVPHYS_PAGES => -1,
        _SC_ATEXIT_MAX => -1,
        _SC_PASS_MAX => -1,
        _SC_XOPEN_VERSION => _XOPEN_VERSION,
        _SC_XOPEN_XCU_VERSION => _XOPEN_VERSION,
        _SC_XOPEN_UNIX => 1,
        _SC_XOPEN_CRYPT => -1,
        _SC_XOPEN_ENH_I18N => 1,
        _SC_XOPEN_SHM => 1,
        _SC_2_CHAR_TERM => -1,
        _SC_2_UPE => -1,
        _SC_XOPEN_XPG2 => -1,
        _SC_XOPEN_XPG3 => -1,
        _SC_XOPEN_XPG4 => -1,
        // TODO: ?
        _SC_NZERO => -1,
        _SC_XBS5_ILP32_OFF32 => -1,
        _SC_XBS5_ILP32_OFFBIG => {
            if size_of::<c_long>() == 4 {
                1
            } else {
                -1
            }
        }
        _SC_XBS5_LP64_OFF64 => {
            if size_of::<c_long>() == 8 {
                1
            } else {
                -1
            }
        }
        _SC_XBS5_LPBIG_OFFBIG => -1,
        _SC_XOPEN_LEGACY => -1,
        _SC_XOPEN_REALTIME => -1,
        _SC_XOPEN_REALTIME_THREADS => -1,
        _SC_ADVISORY_INFO => _POSIX_VERSION,
        _SC_BARRIERS => _POSIX_VERSION,
        _SC_CLOCK_SELECTION => _POSIX_VERSION,
        _SC_CPUTIME => _POSIX_VERSION,
        _SC_THREAD_CPUTIME => _POSIX_VERSION,
        _SC_MONOTONIC_CLOCK => _POSIX_VERSION,
        _SC_READER_WRITER_LOCKS => _POSIX_VERSION,
        _SC_SPIN_LOCKS => _POSIX_VERSION,
        _SC_REGEXP => 1,
        _SC_SHELL => 1,
        _SC_SPAWN => _POSIX_VERSION,
        _SC_SPORADIC_SERVER => -1,
        _SC_THREAD_SPORADIC_SERVER => -1,
        _SC_TIMEOUTS => _POSIX_VERSION,
        _SC_TYPED_MEMORY_OBJECTS => -1,
        _SC_2_PBS => -1,
        _SC_2_PBS_ACCOUNTING => -1,
        _SC_2_PBS_LOCATE => -1,
        _SC_2_PBS_MESSAGE => -1,
        _SC_2_PBS_TRACK => -1,
        // TODO: SYMLOOP_MAX in paths.h
        _SC_SYMLOOP_MAX => -1,
        _SC_STREAMS => 0,
        _SC_2_PBS_CHECKPOINT => -1,
        _SC_V6_ILP32_OFF32 => -1,
        _SC_V6_ILP32_OFFBIG => {
            if size_of::<c_long>() == 4 {
                1
            } else {
                -1
            }
        }
        _SC_V6_LP64_OFF64 => {
            if size_of::<c_long>() == 8 {
                1
            } else {
                -1
            }
        }
        _SC_V6_LPBIG_OFFBIG => -1,
        _SC_HOST_NAME_MAX => HOST_NAME_MAX.try_into().unwrap_or(-1),
        _SC_TRACE => -1,
        _SC_TRACE_EVENT_FILTER => -1,
        _SC_TRACE_INHERIT => -1,
        _SC_TRACE_LOG => -1,
        _SC_IPV6 => _POSIX_VERSION,
        _SC_RAW_SOCKETS => _POSIX_VERSION,
        _SC_V7_ILP32_OFF32 => -1,
        _SC_V7_ILP32_OFFBIG => {
            if size_of::<c_long>() == 4 {
                1
            } else {
                -1
            }
        }
        _SC_V7_LP64_OFF64 => {
            if size_of::<c_long>() == 8 {
                1
            } else {
                -1
            }
        }
        _SC_V7_LPBIG_OFFBIG => -1,
        _SC_SS_REPL_MAX => -1,
        _SC_TRACE_EVENT_NAME_MAX => -1,
        _SC_TRACE_NAME_MAX => -1,
        _SC_TRACE_SYS_MAX => -1,
        _SC_TRACE_USER_EVENT_MAX => -1,
        _SC_XOPEN_STREAMS => 0,
        _SC_THREAD_ROBUST_PRIO_INHERIT => -1,
        _SC_THREAD_ROBUST_PRIO_PROTECT => -1,
        // TODO: Working getauxval
        _SC_MINSIGSTKSZ => -1,
        _SC_SIGSTKSZ => -1,
        _ => {
            platform::ERRNO.set(errno::EINVAL);
            -1
        }
    }
}
