use bitflags::bitflags;

#[derive(Clone, Copy, Debug, Default)]
pub struct ProcMeta {
    pub pid: u32,
    pub pgid: u32,
    pub ppid: u32,
    pub euid: u32,
    pub ruid: u32,
    pub egid: u32,
    pub rgid: u32,
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
}

impl ProcCall {
    pub fn try_from_raw(raw: usize) -> Option<Self> {
        Some(match raw {
            0 => Self::Waitpid,
            1 => Self::Setrens,
            2 => Self::Exit,
            3 => Self::Waitpgid,
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
