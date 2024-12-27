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
}

impl ProcCall {
    pub fn try_from_raw(raw: usize) -> Option<Self> {
        Some(match raw {
            0 => Self::Waitpid,
            1 => Self::Setrens,
            _ => return None,
        })
    }
}
