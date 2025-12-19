//! POSIX header implementations.

pub mod _aio;
pub mod _fenv;
pub mod arpa_inet;
pub mod assert;
pub mod bits_pthread;
pub mod bits_sched;
// complex.h implemented in C
// cpio.h implemented in C
pub mod crypt;
pub mod ctype;
// TODO: curses.h (deprecated)
// TODO: devctl.h
pub mod dirent;
#[path = "dl-tls/mod.rs"]
pub mod dl_tls;
pub mod dlfcn;
pub mod elf;
pub mod endian;
pub mod err;
pub mod errno;
pub mod fcntl;
pub mod float;
// TODO: fmtmsg.h
pub mod fnmatch;
// TODO: ftw.h
pub mod getopt;
pub mod glob;
pub mod grp;
// TODO: iconv.h
pub mod ifaddrs;
pub mod inttypes;
// iso646.h implemented in C
pub mod langinfo;
// TODO: libintl.h
pub mod libgen;
pub mod limits;
pub mod locale;
pub mod malloc;
// math.h implemented in C
pub mod monetary;
// TODO: mqueue.h
// TODO: ndbm.h
pub mod net_if;
pub mod netdb;
pub mod netinet_in;
pub mod netinet_ip;
pub mod netinet_tcp;
// TODO: nl_types.h
// TODO: Remove C header paths.h when cbindgen can export C/Rust strs
// pub mod paths;
pub mod poll;
pub mod pthread;
pub mod pty;
pub mod pwd;
// TODO: re_comp.h (deprecated)
pub mod regex;
// TODO: regexp.h (deprecated)
pub mod sched;
// TODO: search.h
pub mod semaphore;
pub mod setjmp;
pub mod sgtty;
pub mod shadow;
pub mod signal;
// TODO: spawn.h
// TODO: stdalign.h (likely C implementation)
// stdarg.h implemented in C
// stdatomic.h implemented in C
// stdbool.h implemented in C
// stddef.h implemented in C
// stdint.h implemented in C
pub mod stdio;
pub mod stdlib;
// TODO: stdnoreturn.h (likely C implementation)
pub mod string;
pub mod strings;
// TODO: stropts.h (deprecated)
pub mod sys_auxv;
pub mod sys_epoll;
pub mod sys_file;
pub mod sys_ioctl;
// TODO: sys/ipc.h
pub mod sys_mman;
// TODO: sys/msg.h
pub mod sys_ptrace;
pub mod sys_resource;
pub mod sys_select;
// TODO: sys/sem.h
// TODO: sys/shm.h
pub mod sys_socket;
pub mod sys_stat;
pub mod sys_statvfs;
pub mod sys_time;
#[deprecated]
pub mod sys_timeb;
//pub mod sys_times;
pub mod arch_aarch64_user;
pub mod arch_riscv64_user;
pub mod arch_x64_user;
#[cfg(not(target_arch = "x86"))] // TODO: x86
pub mod sys_procfs;
pub mod sys_random;
pub mod sys_syslog;
pub mod sys_types;
pub mod sys_uio;
pub mod sys_un;
pub mod sys_utsname;
pub mod sys_wait;
pub mod tar;
// TODO: term.h (deprecated)
pub mod termios;
// TODO: tgmath.h (likely C implementation)
// TODO: threads.h
pub mod time;
// TODO: uchar.h
// TODO: ucontext.h (deprecated)
// TODO: ulimit.h (deprecated)
// TODO: unctrl.h (deprecated)
pub mod unistd;
#[deprecated]
pub mod utime;
pub mod utmp;
// TODO: utmpx.h
// TODO: varargs.h (deprecated)
pub mod wchar;
pub mod wctype;
// TODO: wordexp.h
// TODO: xti.h (deprecated)
