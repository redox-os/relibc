use crate::{
    header::{fcntl, sys_ioctl, unistd},
    io::{Cursor, Write},
    platform::types::*,
};

pub(super) unsafe fn openpty(name: &mut [u8]) -> Result<(c_int, c_int), ()> {
    //TODO: wrap in auto-close struct
    let master = fcntl::open(c"/dev/ptmx".as_ptr(), fcntl::O_RDWR | fcntl::O_NOCTTY, 0);
    if master < 0 {
        return Err(());
    }

    let mut lock: c_int = 0;
    if sys_ioctl::ioctl(
        master,
        sys_ioctl::TIOCSPTLCK,
        &mut lock as *mut c_int as *mut c_void,
    ) != 0
    {
        unistd::close(master);
        return Err(());
    }

    let mut ptn: c_int = 0;
    if sys_ioctl::ioctl(
        master,
        sys_ioctl::TIOCGPTN,
        &mut ptn as *mut c_int as *mut c_void,
    ) != 0
    {
        unistd::close(master);
        return Err(());
    }

    let mut cursor = Cursor::new(name);
    write!(cursor, "/dev/pts/{}\0", ptn);

    let slave = fcntl::open(
        cursor.get_ref().as_ptr() as *const c_char,
        fcntl::O_RDWR | fcntl::O_NOCTTY,
        0,
    );
    if slave < 0 {
        unistd::close(master);
        return Err(());
    }

    Ok((master, slave))
}
