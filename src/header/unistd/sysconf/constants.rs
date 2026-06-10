// This file holds the sysconf constants with the same values
// across both linux and redox.
// TODO: add more as redox starts to support them.

use crate::platform::types::c_int;

// POSIX.1 {
pub const _SC_ARG_MAX: c_int = 0;
pub const _SC_CHILD_MAX: c_int = 1;
pub const _SC_CLK_TCK: c_int = 2;
pub const _SC_NGROUPS_MAX: c_int = 3;
pub const _SC_OPEN_MAX: c_int = 4;
pub const _SC_STREAM_MAX: c_int = 5;
pub const _SC_TZNAME_MAX: c_int = 6;
// ...
pub const _SC_REALTIME_SIGNALS: c_int = 9; // was 191 on redox
// ...
pub const _SC_TIMERS: c_int = 11;
// ...
pub const _SC_SEMAPHORES: c_int = 21;
pub const _SC_SHARED_MEMORY_OBJECTS: c_int = 22;
// ...
pub const _SC_VERSION: c_int = 29;
pub const _SC_PAGESIZE: c_int = 30;
pub const _SC_PAGE_SIZE: c_int = 30;
// ...
pub const _SC_SIGQUEUE_MAX: c_int = 34; // was 190 on redox
// ...
pub const _SC_RE_DUP_MAX: c_int = 44;
// ...
pub const _SC_THREADS: c_int = 67;
pub const _SC_GETGR_R_SIZE_MAX: c_int = 69;
pub const _SC_GETPW_R_SIZE_MAX: c_int = 70;
pub const _SC_LOGIN_NAME_MAX: c_int = 71;
pub const _SC_TTY_NAME_MAX: c_int = 72;
// ...
pub const _SC_THREAD_ATTR_STACKADDR: c_int = 77;
pub const _SC_THREAD_ATTR_STACKSIZE: c_int = 78;
// ...
pub const _SC_NPROCESSORS_CONF: c_int = 83; // was 57 on redox
pub const _SC_NPROCESSORS_ONLN: c_int = 84; // was 58 on redox
pub const _SC_PHYS_PAGES: c_int = 85; // was 59 on redox
pub const _SC_AVPHYS_PAGES: c_int = 86; // was 60 on redox
// ...
pub const _SC_BARRIERS: c_int = 133;
// ...
pub const _SC_MONOTONIC_CLOCK: c_int = 149;
// ...
pub const _SC_SHELL: c_int = 157;
// ...
pub const _SC_TIMEOUTS: c_int = 164;
// ...
pub const _SC_SYMLOOP_MAX: c_int = 173;
// ...
pub const _SC_HOST_NAME_MAX: c_int = 180;
// ...
// } POSIX.1
