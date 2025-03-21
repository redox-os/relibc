//! getopt implementation for Redox, following http://pubs.opengroup.org/onlinepubs/009695399/functions/getopt.html

use core::ptr;

use crate::{header::getopt, platform::types::*};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getopt.html>.
#[allow(non_upper_case_globals)]
#[no_mangle]
#[linkage = "weak"] // often redefined in GNU programs
pub static mut optarg: *mut c_char = ptr::null_mut();

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getopt.html>.
#[allow(non_upper_case_globals)]
#[no_mangle]
#[linkage = "weak"] // often redefined in GNU programs
pub static mut opterr: c_int = 1;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getopt.html>.
#[allow(non_upper_case_globals)]
#[no_mangle]
#[linkage = "weak"] // often redefined in GNU programs
pub static mut optind: c_int = 1;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getopt.html>.
#[allow(non_upper_case_globals)]
#[no_mangle]
#[linkage = "weak"] // often redefined in GNU programs
pub static mut optopt: c_int = -1;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getopt.html>.
#[no_mangle]
#[linkage = "weak"] // often redefined in GNU programs
pub unsafe extern "C" fn getopt(
    argc: c_int,
    argv: *const *mut c_char,
    optstring: *const c_char,
) -> c_int {
    getopt::getopt_long(argc, argv, optstring, ptr::null(), ptr::null_mut())
}
