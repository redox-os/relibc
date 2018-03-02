use libc;
use syscall;

pub struct stat {
    st_dev: dev_t;     /* ID of device containing file */
    st_ino: ino_t;     /* inode number */
    st_mode: mode_t;    /* protection */
    st_nlink: nlink_t;   /* number of hard links */
    st_uid: uid_t;     /* user ID of owner */
    st_gid: gid_t;     /* group ID of owner */
    st_rdev: dev_t;    /* device ID (if special file) */
    st_size: off_t;    /* total size, in bytes */
    st_blksize: blksize_t; /* blocksize for file system I/O */
    st_blocks: blkcnt_t;  /* number of 512B blocks allocated */
    st_atime: time_t;   /* time of last access */
    st_mtime: time_t;   /* time of last modification */
    st_ctime: time_t;   /* time of last status change */
};

pub struct statvfs {
    f_bsize: libc::c_ulong,
    f_frsize: libc::c_ulong,
    f_blocks: fsblkcnt_t,
    f_bfree: fsblkcnt_t,
    f_bavail: fsblkcnt_t,

    f_files: fsfilcnt_t,
    f_ffree: fsfilcnt_t,
    f_favail: fsfilcnt_t,

    f_fsid: libc::c_ulong,
    f_flag: libc::c_ulong,
    f_namemax: libc::c_ulong,
}

pub struct statfs {
    f_type: __fsword_t,    /* Type of filesystem (see below) */
    f_bsize: __fsword_t,   /* Optimal transfer block size */
    f_blocks: fsblkcnt_t,  /* Total data blocks in filesystem */
    f_bfree: fsblkcnt_t,   /* Free blocks in filesystem */
    f_bavail: fsblkcnt_t,  /* Free blocks available to unprivileged user */
    f_files: fsfilcnt_t,   /* Total file nodes in filesystem */
    f_ffree: fsfilcnt_t,   /* Free file nodes in filesystem */
    f_fsid: fsid_t,    /* Filesystem ID */
    f_namelen: __fsword_t, /* Maximum length of filenames */
    f_frsize: __fsword_t,  /* Fragment size (since Linux 2.6) */
    f_flags: __fsword_t,   /* Mount flags of filesystem  (since Linux 2.6.36) */
    f_spare[6]: __fsword_t, /* Padding bytes reserved for future use */
}

#[repr(C)]
enum Flags {
    ReadOnly = 1,
    NoSUID  = 2
}

pub libc_fn!(stat(c_path: *const libc::c_char, mut c_buf: stat) -> Result<libc::c_int> {
    let path = newlib::cstr_to_slice(c_path);
    let fd = syscall::call::open(path, syscall::O_STAT);
    let mut buf = Stat::default()
    syscall::call::fstat(fd, &buf);
    let _ = syscall::close(fd);
    &c_buf = stat {
        st_dev: &buf.st_dev as dev_t,
        st_ino: &buf.st_ino as ino_t,
        st_mode: &buf.st_mode as mode_t,
        st_nlink: &buf.st_nlink as nlink_t,
        st_uid: &buf.st_uid as uid_t,
        st_gid: &buf.st_gid as gid_t,
        st_rdev: &buf.st_rdev as dev_t,
        st_size: &buf.st_size as off_t,
        st_blksize: &buf.st_blksize as blksize_t,
        st_blocks: &buf.st_blocks as blkcnt_t,
        st_atime: &buf.st_atime as time_t,
        st_mtime: &buf.st_mtime as time_t,
        st_ctime: &buf.st_ctime as time_t,
    };
    Ok(0)
});

pub libcfn!(statvfs(c_path: *const libc::c_char, mut c_buf: statvfs) -> Result<libc::c_int> {
    let path = newlib::cstr_to_slice(c_path);
    let fd = syscall::call::open(path, syscall::O_STAT);
    fstatvfs(fd, c_buf)
});

pub libcfn!(fstatvfs(fd: libc::c_int, mut c_buf: statvfs) -> Result<libc::c_int> {
    let mut buf = StatVfs::default();
    syscall::call::fstatvfs(fd, &buf);
    let _ = syscall::close(fd);
    &c_buf = statvfs {
        f_bsize: &buf.f_bsize as libc::c_ulong,
      //  f_frsize: &buf.f_frsize as libc::c_ulong,
        f_blocks: &buf.f_blocks as fsblkcnt_t,
        f_bavail: &buf.f_bavail as fsblkcnt_t,
      //  f_files: &buf.f_files as fsblkcnt_t,
      //  f_ffree: &buf.f_ffree as fsblkcnt_t,
      //  f_favail: &buf.f_favail as fsblkcnt_t,
      //  f_fsid: &buf.f_fsid as libc::c_ulong,
      //  f_flag: &buf.f_flag as libc::c_ulong,
      //  f_namemax: &buf.f_namemax as libc::c_ulong,
    }
    Ok(0)
});
