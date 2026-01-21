use core::{
    cell::SyncUnsafeCell,
    convert::TryInto,
    mem,
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr,
    str::FromStr,
};

use alloc::{boxed::Box, string::String, vec::Vec};

use crate::{
    c_str::CStr,
    fs::File,
    header::{errno, fcntl, string::strlen},
    io::{BufReader, Lines, prelude::*},
    platform,
    platform::types::{c_char, c_int, c_long, c_ulong, size_t},
};

use super::errno::*;

#[cfg(target_os = "linux")]
const SEPARATOR: char = ':';

#[cfg(target_os = "redox")]
const SEPARATOR: char = ';';

const SHADOW_FILE: &core::ffi::CStr = c"/etc/shadow";

#[derive(Clone, Copy, Debug)]
struct DestBuffer {
    ptr: *mut u8,
    len: usize,
}

#[derive(Debug)]
enum MaybeAllocated {
    Owned(Pin<Box<[u8]>>),
    Borrowed(DestBuffer),
}
impl Deref for MaybeAllocated {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            MaybeAllocated::Owned(boxed) => boxed,
            MaybeAllocated::Borrowed(dst) => unsafe {
                core::slice::from_raw_parts(dst.ptr, dst.len)
            },
        }
    }
}
impl DerefMut for MaybeAllocated {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            MaybeAllocated::Owned(boxed) => boxed,
            MaybeAllocated::Borrowed(dst) => unsafe {
                core::slice::from_raw_parts_mut(dst.ptr, dst.len)
            },
        }
    }
}

static mut SHADOW_BUF: Option<MaybeAllocated> = None;
static mut SHADOW: spwd = spwd {
    sp_namp: ptr::null_mut(),
    sp_pwdp: ptr::null_mut(),
    sp_lstchg: -1,
    sp_min: -1,
    sp_max: -1,
    sp_warn: -1,
    sp_inact: -1,
    sp_expire: -1,
    sp_flag: 0,
};

static LINE_READER: SyncUnsafeCell<Option<Lines<BufReader<File>>>> = SyncUnsafeCell::new(None);

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct spwd {
    pub sp_namp: *mut c_char,
    pub sp_pwdp: *mut c_char,
    pub sp_lstchg: c_long,
    pub sp_min: c_long,
    pub sp_max: c_long,
    pub sp_warn: c_long,
    pub sp_inact: c_long,
    pub sp_expire: c_long,
    pub sp_flag: c_ulong,
}

#[derive(Debug)]
enum Error {
    EOF,
    BufTooSmall,
    Syntax,
}

#[derive(Debug)]
struct OwnedSpwd {
    buffer: MaybeAllocated,
    reference: spwd,
}

impl OwnedSpwd {
    fn into_global(self) -> *mut spwd {
        unsafe {
            SHADOW_BUF = Some(self.buffer);
            SHADOW = self.reference;
            &raw mut SHADOW
        }
    }
}
fn to_long(s: &str) -> c_long {
    c_long::from_str(s).unwrap_or(-1)
}
fn to_ulong(s: &str) -> c_ulong {
    c_ulong::from_str(s).unwrap_or(0)
}

fn parse_spwd(line: String, destbuf: Option<DestBuffer>) -> Result<OwnedSpwd, Error> {
    let mut parts = line.split(SEPARATOR);

    let sp_namp_str = parts.next().ok_or(Error::Syntax)?;
    let sp_pwdp_str = parts.next().ok_or(Error::Syntax)?;

    //TODO: these are not implemented redox-users crate
    let sp_lstchg = to_long(parts.next().unwrap_or(""));
    let sp_min = to_long(parts.next().unwrap_or(""));
    let sp_max = to_long(parts.next().unwrap_or(""));
    let sp_warn = to_long(parts.next().unwrap_or(""));
    let sp_inact = to_long(parts.next().unwrap_or(""));
    let sp_expire = to_long(parts.next().unwrap_or(""));
    let sp_flag = to_ulong(parts.next().unwrap_or(""));

    let string_data_len = sp_namp_str.len() + 1 + sp_pwdp_str.len() + 1;

    let mut buffer = match destbuf {
        Some(buf) => {
            if buf.len < string_data_len {
                platform::ERRNO.set(ERANGE);
                return Err(Error::BufTooSmall);
            }
            MaybeAllocated::Borrowed(buf)
        }
        None => {
            let mut vec = Vec::with_capacity(string_data_len);
            vec.resize(string_data_len, 0);
            MaybeAllocated::Owned(Box::into_pin(vec.into_boxed_slice()))
        }
    };

    let (name_slice, rest) = buffer.split_at_mut(sp_namp_str.len() + 1);
    name_slice[..sp_namp_str.len()].copy_from_slice(sp_namp_str.as_bytes());
    name_slice[sp_namp_str.len()] = 0;

    let (pwd_slice, _) = rest.split_at_mut(sp_pwdp_str.len() + 1);
    pwd_slice[..sp_pwdp_str.len()].copy_from_slice(sp_pwdp_str.as_bytes());
    pwd_slice[sp_pwdp_str.len()] = 0;

    let sp_namp = name_slice.as_mut_ptr() as *mut c_char;
    let sp_pwdp = pwd_slice.as_mut_ptr() as *mut c_char;

    let reference = spwd {
        sp_namp,
        sp_pwdp,
        sp_lstchg,
        sp_min,
        sp_max,
        sp_warn,
        sp_inact,
        sp_expire,
        sp_flag,
    };

    Ok(OwnedSpwd { buffer, reference })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getspnam(name: *const c_char) -> *mut spwd {
    let Ok(db) = File::open(SHADOW_FILE.into(), fcntl::O_RDONLY) else {
        return ptr::null_mut();
    };
    let c_name = unsafe { CStr::from_ptr(name) };

    for line in BufReader::new(db).lines() {
        let Ok(line) = line else { continue };
        if line.starts_with(c_name.to_str().unwrap_or("\0")) {
            if let Ok(pwd) = parse_spwd(line, None) {
                return pwd.into_global();
            }
        }
    }
    ptr::null_mut()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getspnam_r(
    name: *const c_char,
    result_buf: *mut spwd,
    buffer: *mut c_char,
    buflen: size_t,
    result: *mut *mut spwd,
) -> c_int {
    unsafe { *result = ptr::null_mut() };

    let Ok(db) = File::open(SHADOW_FILE.into(), fcntl::O_RDONLY) else {
        return ENOENT;
    };
    let c_name = unsafe { CStr::from_ptr(name).to_str().unwrap_or("\0") };

    for line in BufReader::new(db).lines() {
        let Ok(line) = line else { continue };
        if line.starts_with(c_name) {
            let dest_buf = Some(DestBuffer {
                ptr: buffer as *mut u8,
                len: buflen,
            });
            return match parse_spwd(line, dest_buf) {
                Ok(sp) => {
                    unsafe {
                        *result_buf = sp.reference;
                        *result = result_buf;
                    }
                    0
                }
                Err(Error::BufTooSmall) => ERANGE,
                _ => ENOENT,
            };
        }
    }
    ENOENT
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn setspent() {
    let mut line_reader = unsafe { &mut *LINE_READER.get() };
    if let Ok(db) = File::open(SHADOW_FILE.into(), fcntl::O_RDONLY) {
        *line_reader = Some(BufReader::new(db).lines());
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn endspent() {
    unsafe {
        *LINE_READER.get() = None;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn getspent() -> *mut spwd {
    let line_reader = unsafe { &mut *LINE_READER.get() };
    if line_reader.is_none() {
        unsafe {
            setspent();
        }
    }
    if let Some(lines) = line_reader {
        if let Some(Ok(line)) = lines.next() {
            if let Ok(sp) = parse_spwd(line, None) {
                return sp.into_global();
            }
        }
    }
    ptr::null_mut()
}
