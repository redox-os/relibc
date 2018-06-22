use core::cell::UnsafeCell;
use core::sync::atomic::AtomicBool;
use core::ptr;
use super::{constants, internal, BUFSIZ, FILE, UNGET};

struct GlobalFile(UnsafeCell<FILE>);
impl GlobalFile {
    const fn new(file: FILE) -> Self {
        GlobalFile(UnsafeCell::new(file))
    }
    fn get(&self) -> *mut FILE {
        self.0.get()
    }
}
// statics need to be Sync
unsafe impl Sync for GlobalFile {}

#[allow(non_upper_case_globals)]
static mut default_stdin_buf: [u8; BUFSIZ as usize + UNGET] = [0; BUFSIZ as usize + UNGET];

#[allow(non_upper_case_globals)]
static mut default_stdin: GlobalFile = GlobalFile::new(FILE {
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
});

#[allow(non_upper_case_globals)]
static mut default_stdout_buf: [u8; BUFSIZ as usize] = [0; BUFSIZ as usize];

#[allow(non_upper_case_globals)]
static mut default_stdout: GlobalFile = GlobalFile::new(FILE {
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
});

#[allow(non_upper_case_globals)]
static mut default_stderr_buf: [u8; BUFSIZ as usize] = [0; BUFSIZ as usize];

#[allow(non_upper_case_globals)]
static mut default_stderr: GlobalFile = GlobalFile::new(FILE {
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
});

#[no_mangle]
pub extern "C" fn __stdin() -> *mut FILE {
    unsafe { default_stdin.get() }
}

#[no_mangle]
pub extern "C" fn __stdout() -> *mut FILE {
    unsafe { default_stdout.get() }
}

#[no_mangle]
pub extern "C" fn __stderr() -> *mut FILE {
    unsafe { default_stderr.get() }
}
