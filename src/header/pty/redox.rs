use crate::{
    header::{
        fcntl,
        stdlib::{grantpt, posix_openpt, ptsname_r, unlockpt},
        unistd,
    },
    platform::types::{c_char, c_int},
};

pub(super) unsafe fn openpty(name: &mut [u8]) -> Result<(c_int, c_int), ()> {
    let master = unsafe { posix_openpt(fcntl::O_RDWR | fcntl::O_NOCTTY) };
    if master < 0 {
        return Err(());
    }

    let ret = grantpt(master);
    if ret == -1 {
        unistd::close(master);
        return Err(());
    }
    let ret = unsafe { unlockpt(master) };
    if ret == -1 {
        unistd::close(master);
        return Err(());
    }

    let ret = unsafe { ptsname_r(master, name.as_mut_ptr().cast(), name.len()) };
    if ret < 0 {
        unistd::close(master);
        return Err(());
    }

    let slave = unsafe {
        fcntl::open(
            name.as_ptr() as *const c_char,
            fcntl::O_RDWR | fcntl::O_NOCTTY,
            0,
        )
    };
    if slave < 0 {
        unistd::close(master);
        return Err(());
    }

    Ok((master, slave))
}
