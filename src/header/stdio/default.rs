use super::{BUFSIZ, Buffer, FILE, constants};
use core::{cell::UnsafeCell, ptr};

use crate::{
    fs::File,
    header::pthread,
    io::LineWriter,
    platform::types::c_int,
    sync::{Mutex, Once},
};
use alloc::{boxed::Box, vec::Vec};

// TODO: Change FILE to allow const fn initialization?
pub struct GlobalFile(UnsafeCell<FILE>);

impl GlobalFile {
    fn new(file: c_int, flags: c_int) -> Self {
        let file = File::new(file);
        let writer = Box::new(LineWriter::new(unsafe { file.get_ref() }));
        let mut mutex_attr = pthread::RlctMutexAttr {
            ty: pthread::PTHREAD_MUTEX_RECURSIVE,
            ..Default::default()
        };
        GlobalFile(UnsafeCell::new(FILE {
            lock: pthread::RlctMutex::new(&mutex_attr).unwrap(),

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

// TODO: Allow const fn initialization of FILE
static DEFAULT_STDIN: Once<GlobalFile> = Once::new();
static DEFAULT_STDOUT: Once<GlobalFile> = Once::new();
static DEFAULT_STDERR: Once<GlobalFile> = Once::new();

pub fn default_stdin() -> &'static GlobalFile {
    DEFAULT_STDIN.call_once(|| GlobalFile::new(0, constants::F_NOWR))
}
pub fn default_stdout() -> &'static GlobalFile {
    DEFAULT_STDOUT.call_once(|| GlobalFile::new(1, constants::F_NORD))
}
pub fn default_stderr() -> &'static GlobalFile {
    DEFAULT_STDERR.call_once(|| GlobalFile::new(2, constants::F_NORD))
}

#[unsafe(no_mangle)]
pub static mut stdin: *mut FILE = ptr::null_mut();
#[unsafe(no_mangle)]
pub static mut stdout: *mut FILE = ptr::null_mut();
#[unsafe(no_mangle)]
pub static mut stderr: *mut FILE = ptr::null_mut();
