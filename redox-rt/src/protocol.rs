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
