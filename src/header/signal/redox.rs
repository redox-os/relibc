use core::arch::global_asm;

// Needs to be defined in assembly because it can't have a function prologue
// rax is register, 119 is SIGRETURN
#[cfg(target_arch = "x86_64")]
global_asm!(
    "
    .global __restore_rt
    __restore_rt:
        mov rax, 119
        syscall
"
);
// x8 is register, 119 is SIGRETURN
#[cfg(target_arch = "aarch64")]
global_asm!(
    "
    .global __restore_rt
    __restore_rt:
        mov x8, #119
        svc 0
"
);

pub const SIGHUP: usize = 1;
pub const SIGINT: usize = 2;
pub const SIGQUIT: usize = 3;
pub const SIGILL: usize = 4;
pub const SIGTRAP: usize = 5;
pub const SIGABRT: usize = 6;
pub const SIGBUS: usize = 7;
pub const SIGFPE: usize = 8;
pub const SIGKILL: usize = 9;
pub const SIGUSR1: usize = 10;
pub const SIGSEGV: usize = 11;
pub const SIGUSR2: usize = 12;
pub const SIGPIPE: usize = 13;
pub const SIGALRM: usize = 14;
pub const SIGTERM: usize = 15;
pub const SIGSTKFLT: usize = 16;
pub const SIGCHLD: usize = 17;
pub const SIGCONT: usize = 18;
pub const SIGSTOP: usize = 19;
pub const SIGTSTP: usize = 20;
pub const SIGTTIN: usize = 21;
pub const SIGTTOU: usize = 22;
pub const SIGURG: usize = 23;
pub const SIGXCPU: usize = 24;
pub const SIGXFSZ: usize = 25;
pub const SIGVTALRM: usize = 26;
pub const SIGPROF: usize = 27;
pub const SIGWINCH: usize = 28;
pub const SIGIO: usize = 29;
pub const SIGPWR: usize = 30;
pub const SIGSYS: usize = 31;
pub const NSIG: usize = 32;

pub const SA_NOCLDSTOP: usize = 0x00000001;
pub const SA_NOCLDWAIT: usize = 0x00000002;
pub const SA_SIGINFO: usize = 0x00000004;
pub const SA_RESTORER: usize = 0x04000000;
pub const SA_ONSTACK: usize = 0x08000000;
pub const SA_RESTART: usize = 0x10000000;
pub const SA_NODEFER: usize = 0x40000000;
pub const SA_RESETHAND: usize = 0x80000000;

pub const SS_ONSTACK: usize = 0x00000001;
pub const SS_DISABLE: usize = 0x00000002;

// TODO: It's just a guess based on Linux
pub const MINSIGSTKSZ: usize = 2048;
pub const SIGSTKSZ: usize = 8096;
