//! limits.h implementation for relibc
//! Following https://pubs.opengroup.org/onlinepubs/7908799/xsh/limits.h.html

pub const HOST_NAME_MAX: usize = 64;
pub const NAME_MAX: usize = 255;
pub const PASS_MAX: usize = 128;
pub const PATH_MAX: usize = 4096;
pub const NGROUPS_MAX: usize = 65536;

// TODO: 4096 for most architectures as determined by a quick grep of musl's source; need a better
// way to determine it for other archs or to hard code a value.
#[cfg(target_os = "linux")]
pub const PAGE_SIZE: usize = 4096;

// These POSIX symbols must have these values regardless of OS
pub const _POSIX_AIO_LISTIO_MAX: usize = 2;
pub const _POSIX_AIO_MAX: usize = 1;
pub const _POSIX_ARG_MAX: usize = 4096;
pub const _POSIX_CHILD_MAX: usize = 25;
pub const _POSIX_CLOCKRES_MIN: usize = 20000000;
pub const _POSIX_DELAYTIMER_MAX: usize = 32;
pub const _POSIX_HOST_NAME_MAX: usize = 255;
pub const _POSIX_LINK_MAX: usize = 8;
pub const _POSIX_LOGIN_NAME_MAX: usize = 9;
pub const _POSIX_MAX_CANON: usize = 255;
pub const _POSIX_MAX_INPUT: usize = 255;
pub const _POSIX_NAME_MAX: usize = 14;
pub const _POSIX_NGROUPS_MAX: usize = 8;
pub const _POSIX_OPEN_MAX: usize = 20;
pub const _POSIX_PATH_MAX: usize = 256;
pub const _POSIX_PIPE_BUF: usize = 512;
pub const _POSIX_RE_DUP_MAX: usize = 255;
pub const _POSIX_RTSIG_MAX: usize = 8;
pub const _POSIX_SEM_NSEMS_MAX: usize = 256;
pub const _POSIX_SEM_VALUE_MAX: usize = 32767;
pub const _POSIX_SIGQUEUE_MAX: usize = 32;
pub const _POSIX_SSIZE_MAX: usize = 32767;
pub const _POSIX_STREAM_MAX: usize = 8;
pub const _POSIX_SYMLINK_MAX: usize = 255;
pub const _POSIX_SYMLOOP_MAX: usize = 8;
pub const _POSIX_THREAD_DESTRUCTOR_ITERATIONS: usize = 4;
pub const _POSIX_THREAD_KEYS_MAX: usize = 128;
pub const _POSIX_THREAD_THREADS_MAX: usize = 64;
pub const _POSIX_TIMER_MAX: usize = 32;
pub const _POSIX_TTY_NAME_MAX: usize = 9;
pub const _POSIX_TZNAME_MAX: usize = 6;

pub const _POSIX2_BC_BASE_MAX: usize = 99;
pub const _POSIX2_BC_DIM_MAX: usize = 2048;
pub const _POSIX2_BC_SCALE_MAX: usize = 99;
pub const _POSIX2_BC_STRING_MAX: usize = 1000;
pub const _POSIX2_CHARCLASS_NAME_MAX: usize = 14;
pub const _POSIX2_COLL_WEIGHTS_MAX: usize = 2;
pub const _POSIX2_EXPR_NEST_MAX: usize = 32;
pub const _POSIX2_LINE_MAX: usize = 2048;
pub const _POSIX2_RE_DUP_MAX: usize = 255;
