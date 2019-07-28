use crate::{
    header::errno,
    platform::{self, types::*},
};

pub const _PC_LINK_MAX: c_int = 0;
pub const _PC_MAX_CANON: c_int = 1;
pub const _PC_MAX_INPUT: c_int = 2;
pub const _PC_NAME_MAX: c_int = 3;
pub const _PC_PATH_MAX: c_int = 4;
pub const _PC_PIPE_BUF: c_int = 5;
pub const _PC_CHOWN_RESTRICTED: c_int = 6;
pub const _PC_NO_TRUNC: c_int = 7;
pub const _PC_VDISABLE: c_int = 8;
pub const _PC_SYNC_IO: c_int = 9;
pub const _PC_ASYNC_IO: c_int = 10;
pub const _PC_PRIO_IO: c_int = 11;
pub const _PC_SOCK_MAXBUF: c_int = 12;
pub const _PC_FILESIZEBITS: c_int = 13;
pub const _PC_REC_INCR_XFER_SIZE: c_int = 14;
pub const _PC_REC_MAX_XFER_SIZE: c_int = 15;
pub const _PC_REC_MIN_XFER_SIZE: c_int = 16;
pub const _PC_REC_XFER_ALIGN: c_int = 17;
pub const _PC_ALLOC_SIZE_MIN: c_int = 18;
pub const _PC_SYMLINK_MAX: c_int = 19;
pub const _PC_2_SYMLINKS: c_int = 20;

fn pc(name: c_int) -> c_long {
    // Settings from musl, some adjusted
    match name {
        _PC_LINK_MAX => 127,
        _PC_MAX_CANON => 255,
        _PC_MAX_INPUT => 255,
        _PC_NAME_MAX => 255,
        _PC_PATH_MAX => 4096,
        _PC_PIPE_BUF => 4096,
        _PC_CHOWN_RESTRICTED => 1,
        _PC_NO_TRUNC => 1,
        _PC_VDISABLE => 0,
        _PC_SYNC_IO => 1,
        _PC_ASYNC_IO => -1,
        _PC_PRIO_IO => -1,
        _PC_SOCK_MAXBUF => -1,
        _PC_FILESIZEBITS => 64,
        _PC_REC_INCR_XFER_SIZE => -1,
        _PC_REC_MAX_XFER_SIZE => -1,
        _PC_REC_MIN_XFER_SIZE => 4096,
        _PC_REC_XFER_ALIGN => 4096,
        _PC_ALLOC_SIZE_MIN => 4096,
        _PC_SYMLINK_MAX => -1,
        _PC_2_SYMLINKS => 1,
        _ => {
            unsafe {
                platform::errno = errno::EINVAL;
            }
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn fpathconf(_fildes: c_int, name: c_int) -> c_long {
    pc(name)
}

#[no_mangle]
pub extern "C" fn pathconf(_path: *const c_char, name: c_int) -> c_long {
    pc(name)
}
