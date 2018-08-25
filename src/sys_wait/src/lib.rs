//! sys/wait.h implementation for Redox, following
//! http://pubs.opengroup.org/onlinepubs/7908799/xsh/syswait.h.html

#![no_std]

extern crate platform;
extern crate sys_resource;

use platform::{Pal, Sys};
use platform::types::*;
use sys_resource::rusage;

pub const WNOHANG: c_int = 1;
pub const WUNTRACED: c_int = 2;

pub const WSTOPPED: c_int = 2;
pub const WEXITED: c_int = 4;
pub const WCONTINUED: c_int = 8;
pub const WNOWAIT: c_int = 0x1000000;

pub const __WNOTHREAD: c_int = 0x20000000;
pub const __WALL: c_int = 0x40000000;
pub const __WCLONE: c_int = 0x80000000;

#[no_mangle]
pub unsafe extern "C" fn wait(stat_loc: *mut c_int) -> pid_t {
    waitpid(!0, stat_loc, 0)
}

// #[no_mangle]
pub unsafe extern "C" fn wait3(
    stat_loc: *mut c_int,
    options: c_int,
    resource_usage: *mut rusage,
) -> pid_t {
    unimplemented!();
}

/*
 * TODO: implement idtype_t, id_t, and siginfo_t
 *
 * #[no_mangle]
 * pub unsafe extern "C" fn waitid(
 *     idtype: idtype_t,
 *     id: id_t,
 *     infop: siginfo_t,
 *     options: c_int
 *  ) -> c_int {
 *      unimplemented!();
 *  }
 */

#[no_mangle]
pub unsafe extern "C" fn waitpid(pid: pid_t, stat_loc: *mut c_int, options: c_int) -> pid_t {
    Sys::waitpid(pid, stat_loc, options)
}
