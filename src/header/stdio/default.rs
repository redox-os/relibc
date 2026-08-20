use super::{Buffer, FILE, constants};
use core::cell::UnsafeCell;

use crate::{fs::File, header::pthread, io::LineWriter, platform::types::c_int};
use alloc::vec::Vec;

// TODO: Change FILE to allow const fn initialization?
pub struct GlobalFile(UnsafeCell<FILE>);

impl GlobalFile {
    const fn new(file: c_int, flags: c_int) -> Self {
        let file = File::new(file);
        let writer = LineWriter::new_const(unsafe { file.get_ref() });
        let mutex_attr = pthread::RlctMutexAttr {
            ty: pthread::PTHREAD_MUTEX_RECURSIVE,
            ..pthread::RlctMutexAttr::default_const()
        };
        let Ok(lock) = pthread::RlctMutex::new(&mutex_attr) else {
            unreachable!();
        };
        GlobalFile(UnsafeCell::new(FILE {
            lock,
            file,
            flags: constants::F_PERM | flags,
            read_buf: Buffer::Owned(None),
            read_pos: 0,
            read_size: 0,
            unget: Vec::new(),
            writer: super::FileInnerWriter::Line(writer),

            pid: None,

            orientation: 0,
        }))
    }
    pub const fn get(&self) -> *mut FILE {
        self.0.get()
    }
}
// statics need to be Sync
unsafe impl Sync for GlobalFile {}

static DEFAULT_STDIN: GlobalFile = GlobalFile::new(0, constants::F_NOWR);
static DEFAULT_STDOUT: GlobalFile = GlobalFile::new(1, constants::F_NORD);
static DEFAULT_STDERR: GlobalFile = GlobalFile::new(2, constants::F_NORD);

pub const fn default_stdin() -> &'static GlobalFile {
    &DEFAULT_STDIN
}
pub const fn default_stdout() -> &'static GlobalFile {
    &DEFAULT_STDOUT
}
pub const fn default_stderr() -> &'static GlobalFile {
    &DEFAULT_STDERR
}

#[unsafe(no_mangle)]
pub static mut stdin: *mut FILE = default_stdin().get();
#[unsafe(no_mangle)]
pub static mut stdout: *mut FILE = default_stdout().get();
#[unsafe(no_mangle)]
pub static mut stderr: *mut FILE = default_stderr().get();
