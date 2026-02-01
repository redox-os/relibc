//! `limits.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/limits.h.html>.

use crate::platform::types::{
    c_char, c_int, c_long, c_longlong, c_schar, c_short, c_uchar, c_uint, c_ulong, c_ulonglong,
    c_ushort, ssize_t,
};

pub const HOST_NAME_MAX: usize = 64;
pub const NAME_MAX: usize = 255;
pub const PASS_MAX: usize = 128;
pub const PATH_MAX: usize = 4096;
pub const NGROUPS_MAX: usize = 65536;

pub const CHAR_BIT: u32 = 8;
pub const WORD_BIT: u32 = 32;
#[cfg(target_pointer_width = "32")]
pub const LONG_BIT: u32 = 32;
#[cfg(target_pointer_width = "64")]
pub const LONG_BIT: u32 = 64;

#[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
pub const CHAR_MAX: c_char = 0xFF;
#[cfg(not(any(target_arch = "aarch64", target_arch = "riscv64")))]
pub const CHAR_MAX: c_char = 0x7F;
pub const SCHAR_MAX: c_schar = 0x7F;
pub const SHRT_MAX: c_short = 0x7FFF;
pub const INT_MAX: c_int = 0x7FFF_FFFF;
#[cfg(target_pointer_width = "32")]
pub const LONG_MAX: c_long = 0x7FFF_FFFF;
#[cfg(target_pointer_width = "64")]
pub const LONG_MAX: c_long = 0x7FFF_FFFF_FFFF_FFFF;
pub const LLONG_MAX: c_longlong = 0x7FFF_FFFF_FFFF_FFFF;
#[cfg(target_pointer_width = "32")]
pub const SSIZE_MAX: ssize_t = 0x7FFF_FFFF;
#[cfg(target_pointer_width = "64")]
pub const SSIZE_MAX: ssize_t = 0x7FFF_FFFF_FFFF_FFFF;
pub const UCHAR_MAX: c_uchar = 0xFF;
pub const USHRT_MAX: c_ushort = 0xFFFF;
pub const UINT_MAX: c_uint = 0xFFFFFFFF;

#[cfg(target_pointer_width = "32")]
pub const ULONG_MAX: c_ulong = 0xFFFF_FFFF;
#[cfg(target_pointer_width = "64")]
pub const ULONG_MAX: c_ulong = 0xFFFF_FFFF_FFFF_FFFF;
pub const ULLONG_MAX: c_ulonglong = 0xFFFF_FFFF_FFFF_FFFF;

#[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
pub const CHAR_MIN: c_char = 0;
#[cfg(not(any(target_arch = "aarch64", target_arch = "riscv64")))]
pub const CHAR_MIN: c_char = -0x80;
pub const SCHAR_MIN: c_schar = -SCHAR_MAX - 1;
pub const SHRT_MIN: c_short = -SHRT_MAX - 1;
pub const INT_MIN: c_int = -INT_MAX - 1;
pub const LONG_MIN: c_long = -LONG_MAX - 1;
pub const LLONG_MIN: c_longlong = -LLONG_MAX - 1;

// TODO: 4096 for most architectures as determined by a quick grep of musl's source; need a better
// way to determine it for other archs or to hard code a value.
#[cfg(target_os = "linux")]
pub const PAGE_SIZE: usize = 4096;

// These POSIX symbols must have these values regardless of OS
pub const _POSIX_AIO_LISTIO_MAX: c_long = 2;
pub const _POSIX_AIO_MAX: c_long = 1;
pub const _POSIX_ARG_MAX: c_long = 4096;
pub const _POSIX_CHILD_MAX: c_long = 25;
pub const _POSIX_CLOCKRES_MIN: c_long = 20000000;
pub const _POSIX_DELAYTIMER_MAX: c_long = 32;
pub const _POSIX_HOST_NAME_MAX: c_long = 255;
pub const _POSIX_LINK_MAX: c_long = 8;
pub const _POSIX_LOGIN_NAME_MAX: c_long = 9;
pub const _POSIX_MAX_CANON: c_long = 255;
pub const _POSIX_MAX_INPUT: c_long = 255;
pub const _POSIX_NAME_MAX: c_long = 14;
pub const _POSIX_NGROUPS_MAX: c_long = 8;
pub const _POSIX_OPEN_MAX: c_long = 20;
pub const _POSIX_PATH_MAX: c_long = 256;
pub const _POSIX_PIPE_BUF: c_long = 512;
pub const _POSIX_RE_DUP_MAX: c_long = 255;
pub const _POSIX_RTSIG_MAX: c_long = 8;
pub const _POSIX_SEM_NSEMS_MAX: c_long = 256;
pub const _POSIX_SEM_VALUE_MAX: c_long = 32767;
pub const _POSIX_SIGQUEUE_MAX: c_long = 32;
pub const _POSIX_SSIZE_MAX: c_long = 32767;
pub const _POSIX_STREAM_MAX: c_long = 8;
pub const _POSIX_SYMLINK_MAX: c_long = 255;
pub const _POSIX_SYMLOOP_MAX: c_long = 8;
pub const _POSIX_THREAD_DESTRUCTOR_ITERATIONS: c_long = 4;
pub const _POSIX_THREAD_KEYS_MAX: c_long = 128;
pub const _POSIX_THREAD_THREADS_MAX: c_long = 64;
pub const _POSIX_TIMER_MAX: c_long = 32;
pub const _POSIX_TTY_NAME_MAX: c_long = 9;
pub const _POSIX_TZNAME_MAX: c_long = 6;

pub const _POSIX2_BC_BASE_MAX: c_long = 99;
pub const _POSIX2_BC_DIM_MAX: c_long = 2048;
pub const _POSIX2_BC_SCALE_MAX: c_long = 99;
pub const _POSIX2_BC_STRING_MAX: c_long = 1000;
pub const _POSIX2_CHARCLASS_NAME_MAX: c_long = 14;
pub const _POSIX2_COLL_WEIGHTS_MAX: c_long = 2;
pub const _POSIX2_EXPR_NEST_MAX: c_long = 32;
pub const _POSIX2_LINE_MAX: c_long = 2048;
pub const _POSIX2_RE_DUP_MAX: c_long = 255;

// These symbols must be at least the POSIX values, and sysconf will return the actual value between
// the posix minimum and this maximum.
pub const BC_BASE_MAX: c_long = _POSIX2_BC_BASE_MAX;
pub const BC_DIM_MAX: c_long = _POSIX2_BC_DIM_MAX;
pub const BC_SCALE_MAX: c_long = _POSIX2_BC_SCALE_MAX;
pub const BC_STRING_MAX: c_long = _POSIX2_BC_STRING_MAX;
pub const CHARCLASS_NAME_MAX: c_long = _POSIX2_CHARCLASS_NAME_MAX;
pub const COLL_WEIGHTS_MAX: c_long = _POSIX2_COLL_WEIGHTS_MAX;
pub const EXPR_NEST_MAX: c_long = _POSIX2_EXPR_NEST_MAX;
pub const LINE_MAX: c_long = _POSIX2_LINE_MAX;
pub const RE_DUP_MAX: c_long = _POSIX2_RE_DUP_MAX;

pub const PTHREAD_DESTRUCTOR_ITERATIONS: c_long = _POSIX_THREAD_DESTRUCTOR_ITERATIONS;
// TODO: What should this limit be? Both glibc and musl have it as 1024
pub const PTHREAD_KEYS_MAX: c_long = 4096 * 32;
pub const PTHREAD_STACK_MIN: c_long = 65536;
