use crate::{
    error::ResultExt,
    header::{fcntl, unistd},
    platform::types::*,
    Pal, Sys,
};

pub(super) unsafe fn openpty(name: &mut [u8]) -> Result<(c_int, c_int), ()> {
    let master = fcntl::open(c_str!("/scheme/pty").as_ptr(), fcntl::O_RDWR, 0);
    if master < 0 {
        return Err(());
    }

    // TODO: better error handling
    let count = Sys::fpath(master, name).or_minus_one_errno();
    if count < 0 {
        unistd::close(master);
        return Err(());
    }

    let slave = fcntl::open(name.as_ptr() as *const c_char, fcntl::O_RDWR, 0);
    if slave < 0 {
        unistd::close(master);
        return Err(());
    }

    Ok((master, slave))
}
