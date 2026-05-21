use core::mem;

use crate::{
    error::ResultExt,
    header::{
        errno::EINVAL,
        fcntl::{O_CREAT, O_EXCL, O_RDWR},
        sys_ipc::{IPC_CREAT, IPC_EXCL, IPC_PRIVATE, IPC_RMID, IPC_SET, IPC_STAT, ipc_perm},
        sys_mman::{MAP_SHARED, PROT_READ, PROT_WRITE, shm_open, shm_unlink},
        sys_stat::{fchmod, fstat, stat},
        unistd::ftruncate,
    },
    platform::{
        ERRNO, Pal, Sys,
        types::{c_char, c_int, c_void, key_t, mode_t, pid_t, size_t, time_t},
    },
};

#[allow(non_camel_case_types)]
pub type shmatt_t = core::ffi::c_ushort;

/// Attach read-only (else read-write).
pub const SHM_RDONLY: c_int = 0o10000;
/// Round attach address to SHMLBA.
pub const SHM_RND: c_int = 0o20000;
/// Segment low boundary address multiple.
pub const SHMLBA: size_t = 4096;

/// Return value of `shmat()` indicating shared memory has not been attached.
pub const SHM_FAILED: *mut c_void = -1isize as *mut c_void;

#[repr(C)]
pub struct shmid_ds {
    pub shm_perm: ipc_perm,
    pub shm_segsz: size_t,
    pub shm_lpid: pid_t,
    pub shm_cpid: pid_t,
    pub shm_nattch: shmatt_t,
    pub shm_atime: time_t,
    pub shm_dtime: time_t,
    pub shm_ctime: time_t,
}

// needed because shmdt isn't tracking size
#[derive(Copy, Clone)]
#[repr(C)]
struct ShmHeader {
    total_size: size_t,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/shmget.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shmget(key: key_t, size: size_t, shmflg: c_int) -> c_int {
    let path_str = if key == IPC_PRIVATE {
        format!("/sysv_priv_{}\0", Sys::getpid())
    } else {
        format!("/sysv_key_{}\0", key)
    };

    let path_ptr = path_str.as_ptr().cast::<c_char>();

    let mut oflag = O_RDWR;
    if (shmflg & IPC_CREAT) != 0 {
        oflag |= O_CREAT;
    }
    if (shmflg & IPC_EXCL) != 0 {
        oflag |= O_EXCL;
    }

    let fd = unsafe { shm_open(path_ptr, oflag, 0o666) };
    if fd < 0 {
        return -1;
    }

    if (oflag & O_CREAT) != 0 {
        let total_size = size + mem::size_of::<ShmHeader>();
        if ftruncate(fd, total_size as i64) < 0 {
            return -1;
        }
    }

    unsafe {
        let _ = shm_unlink(path_ptr);
    }

    fd
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/shmat.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shmat(
    shmid: c_int,
    _shmaddr: *const c_void,
    shmflg: c_int,
) -> *mut c_void {
    let mut stat = stat::default();
    if unsafe { fstat(shmid, &raw mut stat) } < 0 {
        return SHM_FAILED;
    }
    let size = stat.st_size as usize;
    let mut prot = PROT_READ;
    if shmflg & SHM_RDONLY == 0 {
        prot |= PROT_WRITE;
    }

    let res = unsafe { Sys::mmap(core::ptr::null_mut(), size, prot, MAP_SHARED, shmid, 0) };
    let ptr = match res {
        Ok(p) => p,
        Err(_) => return SHM_FAILED,
    };

    let header = ptr.cast::<ShmHeader>();
    unsafe {
        (*header).total_size = size;
    }

    unsafe { ptr.add(mem::size_of::<ShmHeader>()) }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/shmdt.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shmdt(shmaddr: *const c_void) -> c_int {
    if shmaddr.is_null() || shmaddr == SHM_FAILED {
        return -1;
    }

    let base_ptr = unsafe { (shmaddr as *mut u8).sub(mem::size_of::<ShmHeader>()) };
    let header = base_ptr.cast::<ShmHeader>();
    let total_size = unsafe { (*header).total_size };

    unsafe { Sys::munmap(base_ptr.cast::<c_void>(), total_size) }
        .map(|_| 0)
        .or_minus_one_errno()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/shmctl.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn shmctl(shmid: c_int, cmd: c_int, buf: *mut shmid_ds) -> c_int {
    match cmd {
        IPC_RMID => Sys::close(shmid).map(|_| 0).or_minus_one_errno(),
        IPC_STAT => {
            if buf.is_null() {
                ERRNO.set(EINVAL);
                return -1;
            }

            let mut stat = stat::default();
            if unsafe { fstat(shmid, &raw mut stat) } < 0 {
                return -1;
            }

            unsafe {
                let buf = &mut *buf;
                buf.shm_segsz =
                    (stat.st_size as size_t).saturating_sub(mem::size_of::<ShmHeader>());
                buf.shm_cpid = stat.st_uid as pid_t;
                buf.shm_lpid = stat.st_gid as pid_t;
                buf.shm_atime = stat.st_atim.tv_sec;
                buf.shm_dtime = stat.st_mtim.tv_sec;
                buf.shm_ctime = stat.st_ctim.tv_sec;
                buf.shm_nattch = stat.st_nlink as shmatt_t;
                buf.shm_perm.uid = stat.st_uid;
                buf.shm_perm.gid = stat.st_gid;
                buf.shm_perm.mode = stat.st_mode & 0o777;
            }
            0
        }
        IPC_SET => {
            if buf.is_null() {
                ERRNO.set(EINVAL);
                return -1;
            }

            let mode = unsafe { (*buf).shm_perm.mode & 0o777 };
            fchmod(shmid, mode as mode_t)
        }
        _ => {
            ERRNO.set(EINVAL);
            -1
        }
    }
}
