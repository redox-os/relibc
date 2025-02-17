use crate::{
    header::{fcntl, sys_ioctl, unistd},
    io::{Cursor, Write},
    platform::types::*,
};

pub(super) unsafe fn openpty(name: &mut [u8]) -> Result<(c_int, c_int), ()> {
    const O_NOCTTY: c_int = 0x100;

    //TODO: wrap in auto-close struct
    let path = c_str!("/dev/ptmx").as_ptr();
    let master = unsafe { fcntl::open(path, fcntl::O_RDWR | O_NOCTTY, 0) };
    if master < 0 {
        return Err(());
    }

    let mut lock: c_int = 0;
    let lock_ptr = &mut lock as *mut c_int as *mut c_void;
    if unsafe { sys_ioctl::ioctl(master, sys_ioctl::TIOCSPTLCK, lock_ptr) } != 0 {
        unistd::close(master);
        return Err(());
    }

    let mut ptn: c_int = 0;
    let ptn_ptr = &mut ptn as *mut c_int as *mut c_void;
    if unsafe { sys_ioctl::ioctl(master, sys_ioctl::TIOCGPTN, ptn_ptr) } != 0 {
        unistd::close(master);
        return Err(());
    }

    let mut cursor = Cursor::new(name);
    write!(cursor, "/dev/pts/{}", ptn);

    let cursor_ptr = cursor.get_ref().as_ptr() as *const c_char;
    let slave = unsafe { fcntl::open(cursor_ptr, fcntl::O_RDWR | O_NOCTTY, 0) };
    if slave < 0 {
        unistd::close(master);
        return Err(());
    }

    Ok((master, slave))
}
