//! sys/wait.h implementation for Redox, following
//! http://pubs.opengroup.org/onlinepubs/7908799/xsh/syswait.h.html

//use header::sys_resource::rusage;
use crate::platform::{types::*, Pal, Sys};

pub const WNOHANG: c_int = 1;
pub const WUNTRACED: c_int = 2;

pub const WSTOPPED: c_int = 2;
pub const WEXITED: c_int = 4;
pub const WCONTINUED: c_int = 8;
pub const WNOWAIT: c_int = 0x0100_0000;

pub const __WNOTHREAD: c_int = 0x2000_0000;
pub const __WALL: c_int = 0x4000_0000;
#[allow(overflowing_literals)]
pub const __WCLONE: c_int = 0x8000_0000;

#[no_mangle]
pub unsafe extern "C" fn wait(stat_loc: *mut c_int) -> pid_t {
    waitpid(!0, stat_loc, 0)
}

// #[no_mangle]
// pub unsafe extern "C" fn wait3(
//     stat_loc: *mut c_int,
//     options: c_int,
//     resource_usage: *mut rusage,
// ) -> pid_t {
//     unimplemented!();
// }

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
