/// unistd implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/unistd.h.html

extern crate libc;

use libc::*;

/*
#[no_mangle]
pub extern "C" fn name(arg) -> c_int {
    unimplemented!();
}
*/

#[no_mangle]
pub extern "C" fn access(pathname: *const c_char, mode: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn alarm(seconds: c_uint) -> c_uint {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn brk(addr: *mut c_void) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn chdir(path: *const c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn chroot(path: *const c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn chown(pathname: *const c_char, owner: uid_t, group: gid_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn close(fd: c_int) -> c_int {
    unimplemented!();
}
