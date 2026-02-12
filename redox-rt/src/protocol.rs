use bitflags::bitflags;

macro_rules! enum_tofrom {
    {
        //$(#[$tl_cfg:meta])*
        $v:vis enum $e:ident : $r:ty {
            $(
                //$($var_cfg:meta)*
                $var:ident = $c:literal
            ),*
            $(,)?
        }
    } => {
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        #[repr($r)]
        //$(#[$tl_cfg])*
        $v enum $e {
            $(
                //$(#[$var_cfg])*
                $var = $c
            ),*
        }
        impl $e {
            pub const fn try_from_raw(raw: $r) -> Option<Self> {
                Some(match raw {
                    $(
                        $c => Self::$var,
                    )*
                    _ => return None,
                })
            }
            pub const fn raw(self) -> $r {
                self as $r
            }
        }
    }
}

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

enum_tofrom! {
    pub enum ProcCall : usize {
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
}
enum_tofrom! {
    pub enum ThreadCall : usize {
        // TODO: replace with sendfd equivalent syscall for sending memory, or force userspace to
        // obtain its TCB memory from this server
        SyncSigTctl = 0,
        SignalThread = 1,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
#[non_exhaustive]
pub enum SocketCall {
    Bind = 0,
    Connect = 1,
    SetSockOpt = 2,
    GetSockOpt = 3,
    SendMsg = 4,
    RecvMsg = 5,
    Unbind = 6,
    GetToken = 7,
    GetPeerName = 8,
    Shutdown = 9,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
#[non_exhaustive]
pub enum FsCall {
    Connect = 0,
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
            6 => Self::Unbind,
            7 => Self::GetToken,
            8 => Self::GetPeerName,
            9 => Self::Shutdown,
            _ => return None,
        })
    }
}

impl FsCall {
    pub fn try_from_raw(raw: usize) -> Option<Self> {
        Some(match raw {
            0 => Self::Connect,
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
            Self::ProcGroup(raw.wrapping_neg() as usize)
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
    pub pid: u32,
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

// TODO: enforce equivalence with consts in header/signal/redox
pub const SI_QUEUE: i32 = -1;
pub const SI_USER: i32 = 0;
pub const SI_TIMER: i32 = 1;
pub const SI_ASYNCIO: i32 = 2;
pub const SI_MESGQ: i32 = 3;

bitflags! {
    #[derive(Clone, Copy, Debug, Default, Eq, Ord, Hash, PartialEq, PartialOrd)]
    pub struct NsPermissions: usize {
        /// List schemes in the namespace
        const LIST = 1 << 0;
        /// Register a new scheme in the namespace
        const INSERT = 1 << 1;
        /// Delete a scheme from the namespace
        const DELETE = 1 << 2;
        /// Get scheme creation capabilities of the namespace
        const SCHEME_CREATE = 1 << 3;
    }
}

enum_tofrom! {
    pub enum NsDup : usize {
        ForkNs = 0,
        ShrinkPermissions = 1,
        IssueRegister = 2,
    }
}

/// Parts of the signal protocol only relevant between procmgr and redox-rt
pub mod signal {
    use super::*;
    use core::sync::atomic::Ordering;
    use syscall::SigProcControl;

    enum_tofrom! {
        // defined to match SI_ consts so compiler just needs to sign extend
        pub enum Code: u8 {
            User = 0,
            Timer = 1,
            AsyncIO = 2,
            Mesgq = 3,
            Queue = 15,
        }
    }
    impl Code {
        pub fn to_si_code(self) -> i32 {
            match self {
                Self::User => SI_USER,
                Self::Timer => SI_TIMER,
                Self::AsyncIO => SI_ASYNCIO,
                Self::Mesgq => SI_MESGQ,
                Self::Queue => SI_QUEUE,
            }
        }
    }

    /// Number of bits used to encode a PID, i.e. `log2(MAX_POSSIBLE_PID+1)`.
    ///
    /// This is fundamentally limited to 31 by being `int` in POSIX, and where -pid as a PGID in some
    /// places. Further, 3 additional bits are reserved so that 4 bits can be used to encode `si_code`
    /// for standard signals.
    pub const PID_BITS: u32 = 28;

    #[derive(Clone, Copy, Debug)]
    pub struct SenderInfo {
        pub pid: u32,
        pub code: Option<Code>,
        pub ruid: u32,
    }
    impl SenderInfo {
        #[inline]
        pub fn raw(self) -> u64 {
            u64::from(self.pid)
                | (u64::from(self.code.map_or(0, Code::raw)) << 28)
                | (u64::from(self.ruid) << 32)
        }
        #[inline]
        pub const fn from_raw(raw: u64) -> Self {
            Self {
                pid: (raw as u32) & ((1 << PID_BITS) - 1),
                code: Code::try_from_raw(((raw as u32) >> PID_BITS) as u8),
                ruid: (raw >> 32) as u32,
            }
        }
    }
    mod private {
        pub trait Sealed {}
    }
    impl private::Sealed for SigProcControl {}

    pub trait SigProcControlExt: private::Sealed {
        /// Checks if `sig` should be ignored based on the current action flags.
        ///
        /// * `sig` - The signal to check (e.g. `SIGCHLD`).
        ///
        /// * `stop_or_continue` - Whether the signal is generated because a child
        /// process stopped (`SIGSTOP`, `SIGTSTP`) or continued (`SIGCONT`). If
        /// `true` and `sig` is `SIGCHLD`, the signal shall not be delivered if the
        /// `SA_NOCLDSTOP` flag is set for `SIGCHLD`.
        fn signal_will_ign(&self, sig: usize, stop_or_continue: bool) -> bool;
        fn signal_will_stop(&self, sig: usize) -> bool;
    }

    impl SigProcControlExt for SigProcControl {
        fn signal_will_ign(&self, sig: usize, stop_or_continue: bool) -> bool {
            let flags = self.actions[sig - 1].first.load(Ordering::Relaxed);
            let will_ign = flags & (1 << 63) != 0;
            let sig_specific = flags & (1 << 62) != 0; // SA_NOCLDSTOP if sig == SIGCHLD

            will_ign || (sig == SIGCHLD && stop_or_continue && sig_specific)
        }
        fn signal_will_stop(&self, sig: usize) -> bool {
            matches!(sig, SIGTSTP | SIGTTIN | SIGTTOU)
                && self.actions[sig - 1].first.load(Ordering::Relaxed) & (1 << 62) != 0
        }
    }
}
