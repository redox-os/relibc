use super::{constants, Buffer, BUFSIZ, FILE};
use core::{cell::UnsafeCell, ptr};

use crate::{fs::File, io::LineWriter, platform::types::*, sync::Mutex};
use alloc::{boxed::Box, vec::Vec};

pub struct GlobalFile(UnsafeCell<FILE>);
impl GlobalFile {
    fn new(file: c_int, flags: c_int) -> Self {
        let file = File::new(file);
        let writer = Box::new(LineWriter::new(unsafe { file.get_ref() }));
        GlobalFile(UnsafeCell::new(FILE {
            lock: Mutex::new(()),

            file,
            flags: constants::F_PERM | flags,
            read_buf: Buffer::Owned(vec![0; BUFSIZ as usize]),
            read_pos: 0,
            read_size: 0,
            unget: Vec::new(),
            writer,

            pid: None,

            orientation: 0,
        }))
    }
    pub fn get(&self) -> *mut FILE {
        self.0.get()
    }
}
// statics need to be Sync
unsafe impl Sync for GlobalFile {}

lazy_static! {
    #[allow(non_upper_case_globals)]
    pub static ref default_stdin: GlobalFile = GlobalFile::new(0, constants::F_NOWR);

    #[allow(non_upper_case_globals)]
    pub static ref default_stdout: GlobalFile = GlobalFile::new(1, constants::F_NORD);

    #[allow(non_upper_case_globals)]
    pub static ref default_stderr: GlobalFile = GlobalFile::new(2, constants::F_NORD);
}

#[no_mangle]
pub static mut stdin: *mut FILE = ptr::null_mut();
#[no_mangle]
pub static mut stdout: *mut FILE = ptr::null_mut();
#[no_mangle]
pub static mut stderr: *mut FILE = ptr::null_mut();
