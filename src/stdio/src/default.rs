use core::sync::atomic::AtomicBool;
use core::ptr;
use super::{internal, BUFSIZ, FILE, UNGET, constants};

#[allow(non_upper_case_globals)]
static mut default_stdin_buf: [u8; BUFSIZ as usize + UNGET] = [0; BUFSIZ as usize + UNGET];

#[allow(non_upper_case_globals)]
static mut default_stdin: FILE = FILE {
    flags: constants::F_PERM | constants::F_NOWR | constants::F_BADJ,
    rpos: ptr::null_mut(),
    rend: ptr::null_mut(),
    wend: ptr::null_mut(),
    wpos: ptr::null_mut(),
    wbase: ptr::null_mut(),
    fd: 0,
    buf: unsafe { &mut default_stdin_buf as *mut [u8] as *mut u8 },
    buf_size: BUFSIZ as usize,
    buf_char: -1,
    unget: UNGET,
    lock: AtomicBool::new(false),
    write: None,
    read: Some(&internal::stdio_read),
    seek: None,
};

#[allow(non_upper_case_globals)]
static mut default_stdout_buf: [u8; BUFSIZ as usize] = [0; BUFSIZ as usize];

#[allow(non_upper_case_globals)]
static mut default_stdout: FILE = FILE {
    flags: constants::F_PERM | constants::F_NORD | constants::F_BADJ,
    rpos: ptr::null_mut(),
    rend: ptr::null_mut(),
    wend: ptr::null_mut(),
    wpos: ptr::null_mut(),
    wbase: ptr::null_mut(),
    fd: 1,
    buf: unsafe { &mut default_stdout_buf } as *mut [u8] as *mut u8,
    buf_size: BUFSIZ as usize,
    buf_char: b'\n' as i8,
    unget: 0,
    lock: AtomicBool::new(false),
    write: Some(&internal::stdio_write),
    read: None,
    seek: None,
};

#[allow(non_upper_case_globals)]
static mut default_stderr_buf: [u8; BUFSIZ as usize] = [0; BUFSIZ as usize];

#[allow(non_upper_case_globals)]
static mut default_stderr: FILE = FILE {
    flags: constants::F_PERM | constants::F_NORD | constants::F_BADJ,
    rpos: ptr::null_mut(),
    rend: ptr::null_mut(),
    wend: ptr::null_mut(),
    wpos: ptr::null_mut(),
    wbase: ptr::null_mut(),
    fd: 2,
    buf: unsafe { &mut default_stderr_buf } as *mut [u8] as *mut u8,
    buf_size: BUFSIZ as usize,
    buf_char: -1,
    unget: 0,
    lock: AtomicBool::new(false),
    write: Some(&internal::stdio_write),
    read: None,
    seek: None,
};

// Don't ask me how the casting below works, I have no idea
// " as *const FILE as *mut FILE" rust pls
//
// -- Tommoa
#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut stdin: *mut FILE = unsafe { &default_stdin } as *const FILE as *mut FILE;

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut stdout: *mut FILE = unsafe { &default_stdout } as *const FILE as *mut FILE;

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut stderr: *mut FILE = unsafe { &default_stderr } as *const FILE as *mut FILE;
