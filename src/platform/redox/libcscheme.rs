use super::libredox;
use crate::{c_str::CStr, header::stdlib::getenv, platform::types::*};
use core::{ptr, slice};
use syscall::{EINVAL, EIO, ENOENT, Error, Result, flag::*};

pub const LIBC_SCHEME: &'static str = "libc:";

const ENV_MAX_LEN: i32 = i32::MAX;

macro_rules! env_str {
    ($lit:expr) => {
        #[allow(unused_unsafe)]
        {
            let val_bytes = unsafe { getenv(concat!($lit, "\0").as_ptr() as *const c_char) };
            if val_bytes != ptr::null_mut() {
                if let Ok(val_str) = unsafe { CStr::from_ptr(val_bytes) }.to_str() {
                    Some(val_str)
                } else {
                    None
                }
            } else {
                None
            }
        }
    };
}

pub fn open(path: &str, flags: usize) -> Result<usize> {
    assert!(path.starts_with(LIBC_SCHEME));

    if flags & O_SYMLINK != 0 {
        return Err(Error::new(ENOENT));
    }

    let basename = match path.strip_prefix(LIBC_SCHEME) {
        Some(path) => path.trim_matches('/'),
        _ => return Err(Error::new(EIO)),
    };

    // Linux seems to allow you to read from or write to any of /dev/{stdin,stderr,stdout}
    match basename {
        "stderr" => syscall::dup(2, &[]),
        "stdin" => syscall::dup(0, &[]),
        "stdout" => syscall::dup(1, &[]),
        "tty" => {
            if let Some(tty) = env_str!("TTY") {
                return redox_rt::sys::open(tty, flags);
            }
            Err(Error::new(ENOENT))
        }
        _ => Err(Error::new(ENOENT)),
    }
}
