//! grp implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/grp.h.html

use core::{
    convert::{TryFrom, TryInto},
    mem,
    ops::{Deref, DerefMut},
    pin::Pin,
    primitive::str,
    ptr, slice, num::ParseIntError, str::Matches,
};

use lazy_static::lazy_static;

use alloc::{borrow::ToOwned, string::{String, FromUtf8Error}};
use libc::strncmp;

use crate::{
    c_str::CStr,
    fs::File,
    header::{errno, fcntl, string::strlen},
    io,
    io::{prelude::*, BufReader, Lines},
    platform::types::*,
    platform,
    sync::Mutex
};

use super::errno::*;

#[derive(Clone, Copy, Debug)]
struct DestBuffer {
    ptr: *mut u8,
    len: usize,
}

// Shamelessly stolen from pwd/mod.rs
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

static mut GROUP_BUF: Option<MaybeAllocated> = None;
static mut GROUP: group = group {
    gr_name: ptr::null_mut(),
    gr_passwd: ptr::null_mut(),
    gr_gid: 0,
    gr_mem: ptr::null_mut(),
};

lazy_static! { static ref LINE_READER: Mutex<Option<Lines<BufReader<File>>>> = Mutex::new(None); }

#[repr(C)]
#[derive(Debug)]
pub struct group {
    pub gr_name: *mut c_char,
    pub gr_passwd: *mut c_char,
    pub gr_gid: gid_t,
    pub gr_mem: *mut *mut c_char,
}

#[derive(Debug)]
enum Error {
    EOF,
    SyntaxError,
    BufTooSmall,
    Misc(io::Error),
    FromUtf8Error(FromUtf8Error),
    ParseIntError(ParseIntError),
    Other
}

#[derive(Debug)]
struct OwnedGrp {
    buffer: MaybeAllocated,
    reference: group,
}

impl OwnedGrp {
    fn into_global(self) -> *mut group {
        unsafe {
            GROUP_BUF = Some(self.buffer);
            GROUP = self.reference;
            &mut GROUP
        }
    }
}

fn split(buf: &mut [u8]) -> Option<group> {
    let gid = match buf[0..mem::size_of::<gid_t>()].try_into() {
        Ok(buf) => gid_t::from_ne_bytes(buf),
        Err(err) => return None
    };
    
    // We moved the gid to the beginning of the byte buffer so we can do this.
    let mut parts = buf[mem::size_of::<gid_t>()..].split_mut(|&c| c == b'\0');
    
    Some(group {
        gr_name: parts.next()?.as_mut_ptr() as *mut i8,
        gr_passwd: parts.next()?.as_mut_ptr() as *mut i8,
        gr_gid: gid,
        gr_mem: parts.next()?.as_mut_ptr() as *mut *mut c_char
        // this will work because this points to the first string, which also happens to be the start of the array. The two are equivalent, just need to by typecast.
    })
}

fn parse_grp(line: String, destbuf: Option<DestBuffer>) -> Result<OwnedGrp, Error> {
    let mut buffer = line.to_owned().into_bytes();

    let mut buffer = buffer
        .into_iter()
        .map(|i| if i == b':' { b'\0' } else { i })
        .chain([b'\0'])
        .collect::<Vec<_>>();
    let mut buffer = buffer
        .split_mut(|i| *i == b'\0');

    let mut gr_gid: gid_t = 0;
    let strings = {
        let mut vec: Vec<u8> = Vec::new();
        
        let gr_name = buffer.next().ok_or(Error::EOF)?.to_vec();
        let gr_passwd = buffer.next().ok_or(Error::EOF)?.to_vec();
        gr_gid = String::from_utf8(buffer.next().ok_or(Error::EOF)?.to_vec())
            .map_err(|err| Error::FromUtf8Error(err))?
            .parse::<gid_t>()
            .map_err(|err| Error::ParseIntError(err))?;
            
        // Place the gid at the beginning of the byte buffer to make getting it back out again later, much faster.
            
        vec.extend(gr_gid.to_ne_bytes());
        vec.extend(gr_name);
        vec.push(0);
        vec.extend(gr_passwd);
        vec.push(0);
        
        for i in buffer.next().ok_or(Error::EOF)?
            .split(|b| *b == b',')
            .filter(|i| i.len() > 0) {
            
            vec.extend(i.to_vec());
            vec.push(0);
        }
        
        vec.extend(0usize.to_ne_bytes());
        
        vec
    };
    
    let mut buffer = match destbuf {
        None => MaybeAllocated::Owned(Box::into_pin(strings.into_boxed_slice())),
        Some(buf) => {
            let mut buf = MaybeAllocated::Borrowed(buf);
            
            if buf.len() < buf.len() {
                unsafe { platform::errno = errno::ERANGE; }
                return Err(Error::BufTooSmall);
            }
            
            buf[..strings.len()].copy_from_slice(&strings);
            buf
        }
    };
    let reference = split(&mut buffer).ok_or(Error::Other)?;
    
    Ok(OwnedGrp {
        buffer,
        reference
    })
}

#[no_mangle]
pub extern "C" fn getgrgid(gid: gid_t) -> *mut group {
    let Ok(db) = File::open(c_str!("/etc/group"), fcntl::O_RDONLY) else { return ptr::null_mut() };

    for line in BufReader::new(db).lines() {
        let Ok(line) = line else { return ptr::null_mut() };
        let Ok(grp) = parse_grp(line, None) else { return ptr::null_mut() };
        
        if grp.reference.gr_gid == gid {
            return grp.into_global();
        }
    }

    return ptr::null_mut();
}

#[no_mangle]
pub extern "C" fn getgrnam(name: *const c_char) -> *mut group {
    let Ok(db) = File::open(c_str!("/etc/group"), fcntl::O_RDONLY) else { return ptr::null_mut() };

    for line in BufReader::new(db).lines() {
        let Ok(line) = line else { return ptr::null_mut() };
        
        let Ok(grp) = parse_grp(line, None) else { return ptr::null_mut() };
        
        // Attempt to prevent BO vulnerabilities
        if unsafe { strncmp(grp.reference.gr_name, name, strlen(grp.reference.gr_name).min(strlen(name))) > 0 } {
            return grp.into_global();
        }
    }

    return ptr::null_mut();
}

#[no_mangle]
pub extern "C" fn getgrgid_r(gid: gid_t, result_buf: *mut group, buffer: *mut c_char, buflen: usize, result: *mut *mut group) -> c_int {
    let Ok(db) = File::open(c_str!("/etc/group"), fcntl::O_RDONLY) else { return ENOENT };

    for line in BufReader::new(db).lines() {
        let Ok(line) = line else { return EINVAL };
        let Ok(mut grp) = parse_grp(line, Some(DestBuffer { ptr: buffer as *mut u8, len: buflen })) else { return EINVAL };

        if grp.reference.gr_gid == gid {
            unsafe {
                *result_buf = grp.reference;
                *result = result_buf;
            };
            
            return 0;
        }
    }
    
    return ENOENT;
}

#[no_mangle]
pub extern "C" fn getgrnam_r(name: *const c_char, result_buf: *mut group, buffer: *mut c_char, buflen: usize, result: *mut *mut group) -> c_int {
    let Ok(db) = File::open(c_str!("/etc/group"), fcntl::O_RDONLY) else { return ENOENT };

    for line in BufReader::new(db).lines() {
        let Ok(line) = line else { return EINVAL };
        let Ok(mut grp) = parse_grp(line, Some(DestBuffer { ptr: buffer as *mut u8, len: buflen })) else { return EINVAL };

        if unsafe { strncmp(grp.reference.gr_name, name, strlen(grp.reference.gr_name).min(strlen(name))) > 0 } {
            unsafe {
                *result_buf = grp.reference;
                *result = result_buf;
            };
            
            return 0;
        }
    }
    
    return ENOENT;
}

#[no_mangle]
pub extern "C" fn getgrent() -> *mut group {
    let mut line_reader = LINE_READER.lock();
    
    if line_reader.is_none() {
        let Ok(db) = File::open(c_str!("/etc/group"), fcntl::O_RDONLY) else { return ptr::null_mut() };
        *line_reader = Some(BufReader::new(db).lines());
    }
    
    if let Some(lines) = line_reader.deref_mut() { 
        let Some(line) = lines.next() else { return ptr::null_mut() };
        let Ok(line) = line else { return ptr::null_mut() };
        
        if let Ok(grp) = parse_grp(line, None) {
            return grp.into_global();
        } else { return ptr::null_mut(); }
        
    } else {
        return ptr::null_mut();
    }
}

#[no_mangle]
pub extern "C" fn endgrent() {
    let mut line_reader = LINE_READER.lock();
    *line_reader = None;
}

#[no_mangle]
pub extern "C" fn setgrent() {
    let mut line_reader = LINE_READER.lock();    
    let Ok(db) = File::open(c_str!("/etc/group"), fcntl::O_RDONLY) else { return };
    *line_reader = Some(BufReader::new(db).lines());
}

#[no_mangle]
pub extern "C" fn getgrouplist(user: *const c_char, group: gid_t, groups: *mut gid_t, ngroups: i32) -> i32 {
    let mut grps = unsafe { Vec::<gid_t>::from_raw_parts(groups, 0, ngroups as usize) };
    let Ok(usr) = (unsafe { crate::c_str::CStr::from_ptr(user).to_str() }) else { return 0 };

    let Ok(db) = File::open(c_str!("/etc/group"), fcntl::O_RDONLY) else { return 0; };

    for line in BufReader::new(db).lines() {
        if grps.len() >= ngroups as usize {
            return ngroups;
        }

        match line {
            Err(_) => return 0,
            Ok(line) => {
                let mut parts = line.split(':');

                let group_name = parts.next().unwrap_or("");
                let group_password = parts.next().unwrap_or("");
                let group_id = parts.next().unwrap_or("-1").parse::<i32>().unwrap();
                let members = parts
                    .next()
                    .unwrap_or("")
                    .split(",")
                    .map(|i| i.trim())
                    .collect::<Vec<_>>();

                if members.iter().any(|i| *i == usr) {
                    grps.push(group_id);
                }
            }
        };
    }

    if grps.len() <= 0 {
        grps.push(group);
    }

    return grps.len() as i32;
}
