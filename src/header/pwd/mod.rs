//! pwd implementation for relibc

use alloc::{boxed::Box, vec::Vec};
use core::{
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr,
};

use crate::{
    fs::File,
    header::{errno, fcntl, string::strcmp},
    io::{prelude::*, BufReader, SeekFrom},
    platform::{self, types::*},
};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "redox")]
mod redox;

#[cfg(target_os = "linux")]
use self::linux as sys;
#[cfg(target_os = "redox")]
use self::redox as sys;

#[repr(C)]
#[derive(Debug)]
pub struct passwd {
    pw_name: *mut c_char,
    pw_passwd: *mut c_char,
    pw_uid: uid_t,
    pw_gid: gid_t,
    pw_gecos: *mut c_char,
    pw_dir: *mut c_char,
    pw_shell: *mut c_char,
}

static mut PASSWD_BUF: Option<MaybeAllocated> = None;
static mut PASSWD: passwd = passwd {
    pw_name: ptr::null_mut(),
    pw_passwd: ptr::null_mut(),
    pw_uid: 0,
    pw_gid: 0,
    pw_gecos: ptr::null_mut(),
    pw_dir: ptr::null_mut(),
    pw_shell: ptr::null_mut(),
};

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

#[derive(Debug)]
struct OwnedPwd {
    buffer: MaybeAllocated,
    reference: passwd,
}

impl OwnedPwd {
    fn into_global(self) -> *mut passwd {
        unsafe {
            PASSWD_BUF = Some(self.buffer);
            PASSWD = self.reference;
            &mut PASSWD
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Cause {
    Eof,
    Other,
}

static mut READER: Option<BufReader<File>> = None;

fn parsed<I, O>(buf: Option<I>) -> Option<O>
where
    I: core::borrow::Borrow<[u8]>,
    O: core::str::FromStr,
{
    let buf = buf?;
    let string = core::str::from_utf8(buf.borrow()).ok()?;
    string.parse().ok()
}

fn getpwent_r(
    reader: &mut BufReader<File>,
    destination: Option<DestBuffer>,
) -> Result<OwnedPwd, Cause> {
    let mut buf = Vec::new();
    if reader
        .read_until(b'\n', &mut buf)
        .map_err(|_| Cause::Other)?
        == 0
    {
        return Err(Cause::Eof);
    }

    // Replace all occurences of ':' with terminating NUL byte
    let mut start = 0;
    while let Some(i) = memchr::memchr(b':', &buf[start..]) {
        buf[start + i] = 0;
        start += i + 1;
    }

    // Place terminating NUL byte at the end, replace newline
    let last = buf.last_mut();
    if last == Some(&mut b'\n') {
        *last.unwrap() = 0;
    } else {
        buf.push(0);
    }

    let mut buf = match destination {
        None => MaybeAllocated::Owned(Box::into_pin(buf.into_boxed_slice())),
        Some(dst) => {
            let mut new = MaybeAllocated::Borrowed(dst);
            if new.len() < buf.len() {
                unsafe {
                    platform::errno = errno::ERANGE;
                }
                return Err(Cause::Other);
            }
            new[..buf.len()].copy_from_slice(&buf);
            new
        }
    };

    // Chop up the result into a valid structure
    let passwd = sys::split(&mut buf).ok_or(Cause::Other)?;

    Ok(OwnedPwd {
        buffer: buf,
        reference: passwd,
    })
}

fn pwd_lookup<F>(mut matches: F, destination: Option<DestBuffer>) -> Result<OwnedPwd, Cause>
where
    F: FnMut(&passwd) -> bool,
{
    let file = match File::open(c_str!("/etc/passwd"), fcntl::O_RDONLY) {
        Ok(file) => file,
        Err(_) => return Err(Cause::Other),
    };

    let mut reader = BufReader::new(file);

    loop {
        let entry = getpwent_r(&mut reader, destination)?;

        if matches(&entry.reference) {
            return Ok(entry);
        }
    }
}

unsafe fn mux(
    status: Result<OwnedPwd, Cause>,
    out: *mut passwd,
    result: *mut *mut passwd,
) -> c_int {
    match status {
        Ok(owned) => {
            *out = owned.reference;
            *result = out;
            0
        }
        Err(Cause::Eof) => {
            *result = ptr::null_mut();
            0
        }
        Err(Cause::Other) => {
            *result = ptr::null_mut();
            -1
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn getpwnam_r(
    name: *const c_char,
    out: *mut passwd,
    buf: *mut c_char,
    size: size_t,
    result: *mut *mut passwd,
) -> c_int {
    mux(
        pwd_lookup(
            |parts| strcmp(parts.pw_name, name) == 0,
            Some(DestBuffer {
                ptr: buf as *mut u8,
                len: size,
            }),
        ),
        out,
        result,
    )
}

#[no_mangle]
pub unsafe extern "C" fn getpwuid_r(
    uid: uid_t,
    out: *mut passwd,
    buf: *mut c_char,
    size: size_t,
    result: *mut *mut passwd,
) -> c_int {
    let slice = core::slice::from_raw_parts_mut(buf as *mut u8, size);
    mux(
        pwd_lookup(
            |part| part.pw_uid == uid,
            Some(DestBuffer {
                ptr: buf as *mut u8,
                len: size,
            }),
        ),
        out,
        result,
    )
}

#[no_mangle]
pub extern "C" fn getpwnam(name: *const c_char) -> *mut passwd {
    pwd_lookup(|parts| unsafe { strcmp(parts.pw_name, name) } == 0, None)
        .map(|res| res.into_global())
        .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn getpwuid(uid: uid_t) -> *mut passwd {
    pwd_lookup(|parts| parts.pw_uid == uid, None)
        .map(|res| res.into_global())
        .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn getpwent() -> *mut passwd {
    let reader = match unsafe { &mut READER } {
        Some(reader) => reader,
        None => {
            let file = match File::open(c_str!("/etc/passwd"), fcntl::O_RDONLY) {
                Ok(file) => file,
                Err(_) => return ptr::null_mut(),
            };
            let reader = BufReader::new(file);
            unsafe {
                READER = Some(reader);
                READER.as_mut().unwrap()
            }
        }
    };
    getpwent_r(reader, None)
        .map(|res| res.into_global())
        .unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn setpwent() {
    if let Some(reader) = unsafe { &mut READER } {
        let _ = reader.seek(SeekFrom::Start(0));
    }
}

#[no_mangle]
pub extern "C" fn endpwent() {
    unsafe {
        READER = None;
    }
}
