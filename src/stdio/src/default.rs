use core::cell::UnsafeCell;
use core::sync::atomic::AtomicBool;
use super::{constants, BUFSIZ, FILE, UNGET};

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

lazy_static! {
    #[allow(non_upper_case_globals)]
    static ref default_stdin: GlobalFile = GlobalFile::new(FILE {
        flags: constants::F_PERM | constants::F_NOWR,
        read: None,
        write: None,
        fd: 0,
        buf: vec![0u8;(BUFSIZ + UNGET) as usize],
        buf_char: -1,
        unget: UNGET,
        lock: AtomicBool::new(false),
    });

    #[allow(non_upper_case_globals)]
    static ref default_stdout: GlobalFile = GlobalFile::new(FILE {
        flags: constants::F_PERM | constants::F_NORD,
        read: None,
        write: None,
        fd: 1,
        buf: vec![0u8;(BUFSIZ + UNGET) as usize],
        buf_char: b'\n' as i8,
        unget: 0,
        lock: AtomicBool::new(false),
    });

    #[allow(non_upper_case_globals)]
    static ref default_stderr: GlobalFile = GlobalFile::new(FILE {
        flags: constants::F_PERM | constants::F_NORD,
        read: None,
        write: None,
        fd: 2,
        buf: vec![0u8;(BUFSIZ + UNGET) as usize],
        buf_char: -1,
        unget: 0,
        lock: AtomicBool::new(false),
    });
}
