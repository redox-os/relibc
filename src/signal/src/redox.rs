#[repr(C)]
pub struct sys_sigset_t {
    pub bits: [u64; 2],
}

#[no_mangle] pub static SIGHUP: usize = 1;
#[no_mangle] pub static SIGINT: usize = 2;
#[no_mangle] pub static SIGQUIT: usize = 3;
#[no_mangle] pub static SIGILL: usize = 4;
#[no_mangle] pub static SIGTRAP: usize = 5;
#[no_mangle] pub static SIGBUS: usize = 7;
#[no_mangle] pub static SIGFPE: usize = 8;
#[no_mangle] pub static SIGKILL: usize = 9;
#[no_mangle] pub static SIGUSR1: usize = 10;
#[no_mangle] pub static SIGSEGV: usize = 11;
#[no_mangle] pub static SIGUSR2: usize = 12;
#[no_mangle] pub static SIGPIPE: usize = 13;
#[no_mangle] pub static SIGALRM: usize = 14;
#[no_mangle] pub static SIGTERM: usize = 15;
#[no_mangle] pub static SIGSTKFLT: usize = 16;
#[no_mangle] pub static SIGCHLD: usize = 17;
#[no_mangle] pub static SIGCONT: usize = 18;
#[no_mangle] pub static SIGSTOP: usize = 19;
#[no_mangle] pub static SIGTSTP: usize = 20;
#[no_mangle] pub static SIGTTIN: usize = 21;
#[no_mangle] pub static SIGTTOU: usize = 22;
#[no_mangle] pub static SIGURG: usize = 23;
#[no_mangle] pub static SIGXCPU: usize = 24;
#[no_mangle] pub static SIGXFSZ: usize = 25;
#[no_mangle] pub static SIGVTALRM: usize = 26;
#[no_mangle] pub static SIGPROF: usize = 27;
#[no_mangle] pub static SIGWINCH: usize = 28;
#[no_mangle] pub static SIGIO: usize = 29;
#[no_mangle] pub static SIGPWR: usize = 30;
#[no_mangle] pub static SIGSYS: usize = 31;

#[no_mangle] pub static SA_NOCLDSTOP: usize = 0x00000001;
#[no_mangle] pub static SA_NOCLDWAIT: usize = 0x00000002;
#[no_mangle] pub static SA_SIGINFO: usize = 0x00000004;
#[no_mangle] pub static SA_RESTORER: usize = 0x04000000;
#[no_mangle] pub static SA_ONSTACK: usize = 0x08000000;
#[no_mangle] pub static SA_RESTART: usize = 0x10000000;
#[no_mangle] pub static SA_NODEFER: usize = 0x40000000;
#[no_mangle] pub static SA_RESETHAND: usize = 0x80000000;
