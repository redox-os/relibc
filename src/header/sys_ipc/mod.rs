use crate::{
    c_str::CStr,
    error::ResultExt,
    header::sys_stat::stat,
    out::Out,
    platform::{
        Pal, Sys,
        types::{c_char, c_int, c_ushort, gid_t, key_t, mode_t, uid_t},
    },
};

pub const IPC_R: u32 = 0o400;
pub const IPC_W: u32 = 0o200;
pub const IPC_M: u32 = 0o10000;

/// Remove identifier.
pub const IPC_RMID: i32 = 0;
/// Set options.
pub const IPC_SET: i32 = 1;
/// Get options.
pub const IPC_STAT: i32 = 2;
// pub const IPC_INFO: i32 = 3; non posix unimplemented

/// Create entry if key does not exist.
pub const IPC_CREAT: i32 = 0o1000;
/// Fail if key exists.
pub const IPC_EXCL: i32 = 0o2000;
/// Error if request would need to wait.
pub const IPC_NOWAIT: i32 = 0o4000;

/// Private key.
pub const IPC_PRIVATE: key_t = 0;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ipc_perm {
    pub __key: key_t,
    pub uid: uid_t,
    pub gid: gid_t,
    pub cuid: uid_t,
    pub cgid: gid_t,
    pub mode: mode_t,
    pub __seq: c_ushort,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/ftok.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ftok(path: *const c_char, id: c_int) -> key_t {
    let path = unsafe { CStr::from_ptr(path) };
    let mut stat = stat::default();
    if Sys::stat(path, Out::from_mut(&mut stat))
        .map(|()| 0)
        .or_minus_one_errno()
        == -1
    {
        return -1;
    }

    // Borrowed from musl
    (stat.st_ino & 0xffff) as key_t | ((stat.st_dev & 0xff) << 16) as key_t | ((id & 0xff) << 24)
}
