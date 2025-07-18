use bitflags::bitflags;

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct ProcMeta {
    pub pid: u32,
    pub pgid: u32,
    pub ppid: u32,
    pub ruid: u32,
    pub euid: u32,
    pub suid: u32,
    pub rgid: u32,
    pub egid: u32,
    pub sgid: u32,
    pub ens: u32,
    pub rns: u32,
}
unsafe impl plain::Plain for ProcMeta {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum ProcCall {
    Waitpid = 0,
    Setrens = 1,
    Exit = 2,
    Waitpgid = 3,
    SetResugid = 4,
    Setpgid = 5,
    Getsid = 6,
    Setsid = 7,
    Kill = 8,
    Sigq = 9,

    // TODO: replace with sendfd equivalent syscall for sending memory
    SyncSigPctl = 10,
    Sigdeq = 11,
    Getppid = 12,
    Rename = 13,
    DisableSetpgid = 14,

    // Temporary calls for getting process credentials
    GetProcCredentials = 15,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum ThreadCall {
    // TODO: replace with sendfd equivalent syscall for sending memory, or force userspace to
    // obtain its TCB memory from this server
    SyncSigTctl = 0,
    SignalThread = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum SocketCall {
    Bind = 0,
    Connect = 1,
    SetSockOpt = 2,
    GetSockOpt = 3,
    SendMsg = 4,
    RecvMsg = 5,
}

impl ProcCall {
    pub fn try_from_raw(raw: usize) -> Option<Self> {
        Some(match raw {
            0 => Self::Waitpid,
            1 => Self::Setrens,
            2 => Self::Exit,
            3 => Self::Waitpgid,
            4 => Self::SetResugid,
            5 => Self::Setpgid,
            6 => Self::Getsid,
            7 => Self::Setsid,
            8 => Self::Kill,
            9 => Self::Sigq,
            10 => Self::SyncSigPctl,
            11 => Self::Sigdeq,
            12 => Self::Getppid,
            13 => Self::Rename,
            14 => Self::DisableSetpgid,
            15 => Self::GetProcCredentials,
            _ => return None,
        })
    }
}
impl ThreadCall {
    pub fn try_from_raw(raw: usize) -> Option<Self> {
        Some(match raw {
            0 => Self::SyncSigTctl,
            1 => Self::SignalThread,
            _ => return None,
        })
    }
}

impl SocketCall {
    pub fn try_from_raw(raw: usize) -> Option<Self> {
        Some(match raw {
            0 => Self::Bind,
            1 => Self::Connect,
            2 => Self::SetSockOpt,
            3 => Self::GetSockOpt,
            4 => Self::SendMsg,
            5 => Self::RecvMsg,
            _ => return None,
        })
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, Default, Eq, Ord, Hash, PartialEq, PartialOrd)]
    pub struct WaitFlags: usize {
        const WNOHANG =    0x01;
        const WUNTRACED =  0x02;
        const WCONTINUED = 0x08;
    }
}
/// True if status indicates the child is stopped.
pub fn wifstopped(status: usize) -> bool {
    (status & 0xff) == 0x7f
}

/// If wifstopped(status), the signal that stopped the child.
pub fn wstopsig(status: usize) -> usize {
    (status >> 8) & 0xff
}

/// True if status indicates the child continued after a stop.
pub fn wifcontinued(status: usize) -> bool {
    status == 0xffff
}

/// True if STATUS indicates termination by a signal.
pub fn wifsignaled(status: usize) -> bool {
    ((status & 0x7f) + 1) as i8 >= 2
}

/// If wifsignaled(status), the terminating signal.
pub fn wtermsig(status: usize) -> usize {
    status & 0x7f
}

/// True if status indicates normal termination.
pub fn wifexited(status: usize) -> bool {
    wtermsig(status) == 0
}

/// If wifexited(status), the exit status.
pub fn wexitstatus(status: usize) -> usize {
    (status >> 8) & 0xff
}

/// True if status indicates a core dump was created.
pub fn wcoredump(status: usize) -> bool {
    (status & 0x80) != 0
}
#[derive(Clone, Copy, Debug)]
pub enum ProcKillTarget {
    ThisGroup,
    SingleProc(usize),
    ProcGroup(usize),
    All,
}
impl ProcKillTarget {
    pub fn raw(self) -> usize {
        match self {
            Self::ThisGroup => 0,
            Self::SingleProc(p) => p,
            Self::ProcGroup(g) => usize::wrapping_neg(g),
            Self::All => usize::wrapping_neg(1),
        }
    }
    pub fn from_raw(raw: usize) -> Self {
        let raw = raw as isize;
        if raw == 0 {
            Self::ThisGroup
        } else if raw == -1 {
            Self::All
        } else if raw < 0 {
            Self::ProcGroup(raw as usize)
        } else {
            Self::SingleProc(raw as usize)
        }
    }
}
#[derive(Copy, Clone, Debug, Default, PartialEq)]
#[repr(C)]
pub struct RtSigInfo {
    pub arg: usize,
    pub code: i32,
    pub uid: u32,
    pub pid: u32, // TODO: usize?
}
unsafe impl plain::Plain for RtSigInfo {}

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
pub const SIGCHLD: usize = syscall::SIGCHLD;
pub const SIGCONT: usize = 18;
pub const SIGSTOP: usize = 19;
pub const SIGTSTP: usize = syscall::SIGTSTP;
pub const SIGTTIN: usize = syscall::SIGTTIN;
pub const SIGTTOU: usize = syscall::SIGTTOU;
pub const SIGURG: usize = 23;
pub const SIGXCPU: usize = 24;
pub const SIGXFSZ: usize = 25;
pub const SIGVTALRM: usize = 26;
pub const SIGPROF: usize = 27;
pub const SIGWINCH: usize = 28;
pub const SIGIO: usize = 29;
pub const SIGPWR: usize = 30;
pub const SIGSYS: usize = 31;
