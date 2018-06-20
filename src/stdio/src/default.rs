use core::sync::atomic::AtomicBool;
use super::{constants, BUFSIZ, FILE, UNGET};

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref default_stdin: FILE = FILE {
        flags: constants::F_PERM | constants::F_NOWR | constants::F_BADJ,
        read: None,
        write: None,
        fd: 0,
        buf: vec![0u8;(BUFSIZ + UNGET) as usize],
        buf_char: -1,
        unget: UNGET,
        lock: AtomicBool::new(false),
    };

    #[allow(non_upper_case_globals)]
    static ref default_stdout: FILE = FILE {
        flags: constants::F_PERM | constants::F_NORD | constants::F_BADJ,
        read: None,
        write: None,
        fd: 1,
        buf: vec![0u8;(BUFSIZ + UNGET) as usize],
        buf_char: b'\n' as i8,
        unget: 0,
        lock: AtomicBool::new(false),
    };

    #[allow(non_upper_case_globals)]
    static ref default_stderr: FILE = FILE {
        flags: constants::F_PERM | constants::F_NORD | constants::F_BADJ,
        read: None,
        write: None,
        fd: 2,
        buf: vec![0u8;(BUFSIZ + UNGET) as usize],
        buf_char: -1,
        unget: 0,
        lock: AtomicBool::new(false),
    };
}

// Don't ask me how the casting below works, I have no idea
// " as *const FILE as *mut FILE" rust pls
//
// -- Tommoa
#[no_mangle]
pub static mut stdin: *mut FILE = &default_stdin as *const _ as *const FILE as *mut FILE;

#[no_mangle]
pub static mut stdout: *mut FILE = &default_stdout as *const _ as *const FILE as *mut FILE;

#[no_mangle]
pub static mut stderr: *mut FILE = &default_stderr as *const _ as *const FILE as *mut FILE;
