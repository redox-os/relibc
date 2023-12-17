pub mod _aio;
pub mod arpa_inet;
pub mod assert;
pub mod bits_pthread;
pub mod bits_sched;
pub mod ctype;
pub mod dirent;
#[path = "dl-tls/mod.rs"]
pub mod dl_tls;
pub mod dlfcn;
pub mod elf;
pub mod errno;
pub mod fcntl;
pub mod float;
pub mod fnmatch;
pub mod getopt;
pub mod grp;
pub mod inttypes;
pub mod libgen;
pub mod limits;
pub mod locale;
pub mod netdb;
pub mod netinet_in;
pub mod netinet_ip;
pub mod netinet_tcp;
pub mod poll;
pub mod pthread;
pub mod pwd;
pub mod regex;
pub mod sched;
pub mod semaphore;
pub mod setjmp;
pub mod sgtty;
pub mod signal;
pub mod stdio;
pub mod stdlib;
pub mod string;
pub mod strings;
pub mod sys_auxv;
pub mod sys_epoll;
pub mod sys_file;
pub mod sys_ioctl;
pub mod sys_mman;
pub mod sys_ptrace;
pub mod sys_resource;
pub mod sys_select;
pub mod sys_socket;
pub mod sys_stat;
pub mod sys_statvfs;
pub mod sys_time;
pub mod sys_timeb;
//pub mod sys_times;
pub mod arch_aarch64_user;
pub mod arch_x64_user;
#[cfg(not(target_arch = "x86"))] // TODO: x86
pub mod sys_procfs;
pub mod sys_random;
pub mod sys_types;
pub mod sys_uio;
pub mod sys_un;
pub mod sys_utsname;
pub mod sys_wait;
pub mod termios;
pub mod time;
pub mod unistd;
pub mod utime;
pub mod wchar;
pub mod wctype;
