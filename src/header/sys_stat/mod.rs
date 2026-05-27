//! `sys/stat.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_stat.h.html>.

use crate::{
    c_str::CStr,
    error::ResultExt,
    header::{fcntl::AT_SYMLINK_NOFOLLOW, time::timespec},
    out::Out,
    platform::{
        Pal, Sys,
        types::{
            blkcnt_t, blksize_t, c_char, c_int, c_long, dev_t, gid_t, ino_t, mode_t, nlink_t,
            off_t, uid_t,
        },
    },
};

/// Type of file.
pub const S_IFMT: mode_t = 0o0_170_000;

/// Directory.
pub const S_IFDIR: mode_t = 0o040_000;
/// Character special.
pub const S_IFCHR: mode_t = 0o020_000;
/// Block special.
pub const S_IFBLK: mode_t = 0o060_000;
/// Regular.
pub const S_IFREG: mode_t = 0o100_000;
/// FIFO special.
pub const S_IFIFO: mode_t = 0o010_000;
/// Symbolic link.
pub const S_IFLNK: mode_t = 0o120_000;
/// Socket.
pub const S_IFSOCK: mode_t = 0o140_000;

/// Read, write, execute/search by owner.
pub const S_IRWXU: mode_t = 0o0_700;
/// Read permission, owner.
pub const S_IRUSR: mode_t = 0o0_400;
/// Write permission, owner.
pub const S_IWUSR: mode_t = 0o0_200;
/// Execute/search permission, owner.
pub const S_IXUSR: mode_t = 0o0_100;

// Defined for compatibility
pub const S_IREAD: mode_t = S_IRUSR;
pub const S_IWRITE: mode_t = S_IWUSR;
pub const S_IEXEC: mode_t = S_IXUSR;

/// Read, write, execute/search by group.
pub const S_IRWXG: mode_t = 0o0_070;
/// Read permission, group.
pub const S_IRGRP: mode_t = 0o0_040;
/// Write permission, group.
pub const S_IWGRP: mode_t = 0o0_020;
/// Execute/search permission, group.
pub const S_IXGRP: mode_t = 0o0_010;

/// Read, write, execute/search by others.
pub const S_IRWXO: mode_t = 0o0_007;
/// Read permission, others.
pub const S_IROTH: mode_t = 0o0_004;
/// Write permission, others.
pub const S_IWOTH: mode_t = 0o0_002;
/// Execute/search permission, others.
pub const S_IXOTH: mode_t = 0o0_001;
/// Set-user-ID on execution.
pub const S_ISUID: mode_t = 0o4_000;
/// Set-group-ID on execution.
pub const S_ISGID: mode_t = 0o2_000;
/// On directories, restricted deletion flag.
pub const S_ISVTX: mode_t = 0o1_000;

pub const UTIME_NOW: c_long = (1 << 30) - 1;
pub const UTIME_OMIT: c_long = (1 << 30) - 2;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_stat.h.html>.
#[repr(C)]
#[derive(Default)]
pub struct stat {
    /// Device ID of device containing file.
    pub st_dev: dev_t,
    /// File serial number.
    pub st_ino: ino_t,
    /// Number of hard links to the file.
    pub st_nlink: nlink_t,
    /// Mode of file.
    pub st_mode: mode_t,
    /// User ID of file.
    pub st_uid: uid_t,
    /// Group ID of file.
    pub st_gid: gid_t,
    /// Device ID (if file is character or block special).
    pub st_rdev: dev_t,
    /// For regular files, the file size in bytes.
    /// For symbolic links, the length in bytes of the pathname contained
    /// in the symbolic link.
    /// For a shared or typed memory object, the length in bytes.
    /// For other file types, the use of this field is unspecified.
    pub st_size: off_t,
    /// A file system-specific preferred I/O block size for this object.
    /// In some file system types, this may vary from file to file.
    pub st_blksize: blksize_t,
    /// Number of blocks allocated for this object.
    pub st_blocks: blkcnt_t,

    /// Last data access timestamp.
    pub st_atim: timespec,
    /// Last data modification timestamp.
    pub st_mtim: timespec,
    /// Last file status change timestamp.
    pub st_ctim: timespec,

    // Compared to glibc, our struct is for some reason 24 bytes too small.
    // Accessing atime works, so clearly the struct isn't incorrect...
    // This works.
    pub _pad: [c_char; 24],
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/chmod.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn chmod(path: *const c_char, mode: mode_t) -> c_int {
    let path = unsafe { CStr::from_ptr(path) };
    Sys::chmod(path, mode).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fchmod.html>.
#[unsafe(no_mangle)]
pub extern "C" fn fchmod(fildes: c_int, mode: mode_t) -> c_int {
    Sys::fchmod(fildes, mode).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fchmodat.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fchmodat(
    dirfd: c_int,
    path: *const c_char,
    mode: mode_t,
    flags: c_int,
) -> c_int {
    let path = unsafe { CStr::from_nullable_ptr(path) };
    Sys::fchmodat(dirfd, path, mode, flags)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fstat.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fstat(fildes: c_int, buf: *mut stat) -> c_int {
    let buf = unsafe { Out::nonnull(buf) };
    Sys::fstat(fildes, buf).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/fstatat.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fstatat(
    fildes: c_int,
    path: *const c_char,
    buf: *mut stat,
    flags: c_int,
) -> c_int {
    let path = unsafe { CStr::from_nullable_ptr(path) };
    let buf = unsafe { Out::nonnull(buf) };
    Sys::fstatat(fildes, path, buf, flags)
        .map(|()| 0)
        .or_minus_one_errno()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __fxstat(_ver: c_int, fildes: c_int, buf: *mut stat) -> c_int {
    unsafe { fstat(fildes, buf) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/futimens.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn futimens(fd: c_int, times: *const timespec) -> c_int {
    unsafe { Sys::futimens(fd, times) }
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/lstat.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lstat(path: *const c_char, buf: *mut stat) -> c_int {
    let path = unsafe { CStr::from_ptr(path) };
    let buf = unsafe { Out::nonnull(buf) };
    Sys::lstat(path, buf).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mkdirat.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mkdirat(dirfd: c_int, path: *const c_char, mode: mode_t) -> c_int {
    let path = unsafe { CStr::from_ptr(path) };
    Sys::mkdirat(dirfd, path, mode)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mkdir.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mkdir(path: *const c_char, mode: mode_t) -> c_int {
    let path = unsafe { CStr::from_ptr(path) };
    Sys::mkdir(path, mode).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mkfifoat.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mkfifoat(dirfd: c_int, path: *const c_char, mode: mode_t) -> c_int {
    let path = unsafe { CStr::from_ptr(path) };
    Sys::mkfifoat(dirfd, path, mode)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mkfifo.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mkfifo(path: *const c_char, mode: mode_t) -> c_int {
    let path = unsafe { CStr::from_ptr(path) };
    Sys::mkfifo(path, mode).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mknod.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mknod(path: *const c_char, mode: mode_t, dev: dev_t) -> c_int {
    let path = unsafe { CStr::from_ptr(path) };
    Sys::mknod(path, mode, dev).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/mknodat.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mknodat(
    dirfd: c_int,
    path: *const c_char,
    mode: mode_t,
    dev: dev_t,
) -> c_int {
    let path = unsafe { CStr::from_ptr(path) };
    Sys::mknodat(dirfd, path, mode, dev)
        .map(|()| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/stat.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn stat(file: *const c_char, buf: *mut stat) -> c_int {
    let file = unsafe { CStr::from_ptr(file) };
    let buf = unsafe { Out::nonnull(buf) };
    Sys::stat(file, buf).map(|()| 0).or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/umask.html>.
#[unsafe(no_mangle)]
pub extern "C" fn umask(mask: mode_t) -> mode_t {
    Sys::umask(mask)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/utimensat.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utimensat(
    fd: c_int,
    path: *const c_char,
    times: *const timespec,
    flag: c_int,
) -> c_int {
    let path = unsafe { CStr::from_ptr(path) };
    match Sys::openat(fd, path, flag & AT_SYMLINK_NOFOLLOW, 0) {
        Ok(fd) => unsafe {
            let r = Sys::futimens(fd, times).map(|()| 0).or_minus_one_errno();
            let _ = Sys::close(fd);
            r
        },
        r => r.or_minus_one_errno(),
    }
}
