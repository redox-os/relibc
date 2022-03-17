use crate::{
    c_str::{CStr, CString},
    header::{fcntl, unistd},
    platform::{types::*, Pal, Sys},
};

pub use self::sys::*;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod sys;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;

pub const MAP_SHARED: c_int = 0x0001;
pub const MAP_PRIVATE: c_int = 0x0002;
pub const MAP_TYPE: c_int = 0x000F;
pub const MAP_ANON: c_int = 0x0020;
pub const MAP_ANONYMOUS: c_int = MAP_ANON;

pub const MS_ASYNC: c_int = 0x0001;
pub const MS_INVALIDATE: c_int = 0x0002;
pub const MS_SYNC: c_int = 0x0004;

#[no_mangle]
pub unsafe extern "C" fn mlock(addr: *const c_void, len: usize) -> c_int {
    Sys::mlock(addr, len)
}

#[no_mangle]
pub extern "C" fn mlockall(flags: c_int) -> c_int {
    Sys::mlockall(flags)
}

#[no_mangle]
pub unsafe extern "C" fn mmap(
    addr: *mut c_void,
    len: size_t,
    prot: c_int,
    flags: c_int,
    fildes: c_int,
    off: off_t,
) -> *mut c_void {
    Sys::mmap(addr, len, prot, flags, fildes, off)
}

#[no_mangle]
pub unsafe extern "C" fn mprotect(addr: *mut c_void, len: size_t, prot: c_int) -> c_int {
    Sys::mprotect(addr, len, prot)
}

#[no_mangle]
pub unsafe extern "C" fn msync(addr: *mut c_void, len: size_t, flags: c_int) -> c_int {
    Sys::msync(addr, len, flags)
}

#[no_mangle]
pub unsafe extern "C" fn munlock(addr: *const c_void, len: usize) -> c_int {
    Sys::munlock(addr, len)
}

#[no_mangle]
pub extern "C" fn munlockall() -> c_int {
    Sys::munlockall()
}

#[no_mangle]
pub unsafe extern "C" fn munmap(addr: *mut c_void, len: size_t) -> c_int {
    Sys::munmap(addr, len)
}

#[cfg(target_os = "linux")]
static SHM_PATH: &'static [u8] = b"/dev/shm/";

#[cfg(target_os = "redox")]
static SHM_PATH: &'static [u8] = b"shm:";

unsafe fn shm_path(name: *const c_char) -> CString {
    let name_c = CStr::from_ptr(name);

    let mut path = SHM_PATH.to_vec();

    let mut skip_slash = true;
    for &b in name_c.to_bytes() {
        if skip_slash {
            if b == b'/' {
                continue;
            } else {
                skip_slash = false;
            }
        }
        path.push(b);
    }

    CString::from_vec_unchecked(path)
}

#[no_mangle]
pub unsafe extern "C" fn shm_open(name: *const c_char, oflag: c_int, mode: mode_t) -> c_int {
    let path = shm_path(name);
    fcntl::sys_open(path.as_ptr(), oflag, mode)
}

#[no_mangle]
pub unsafe extern "C" fn shm_unlink(name: *const c_char) -> c_int {
    let path = shm_path(name);
    unistd::unlink(path.as_ptr())
}
