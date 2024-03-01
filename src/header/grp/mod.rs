//! grp implementation, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/grp.h.html

use core::{
    cell::SyncUnsafeCell,
    convert::{TryFrom, TryInto},
    mem::{self, MaybeUninit},
    num::ParseIntError,
    ops::{Deref, DerefMut},
    pin::Pin,
    primitive::str,
    ptr, slice,
    str::Matches,
};

use alloc::{
    borrow::ToOwned,
    string::{FromUtf8Error, String},
};

use crate::{
    c_str::CStr,
    fs::File,
    header::{errno, fcntl, limits, string::strlen, unistd},
    io,
    io::{prelude::*, BufReader, Lines},
    platform,
    platform::types::*,
    sync::Mutex,
};

use super::{errno::*, string::strncmp};

#[cfg(target_os = "linux")]
const SEPARATOR: char = ':';

#[cfg(target_os = "redox")]
const SEPARATOR: char = ';';

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

static LINE_READER: SyncUnsafeCell<Option<Lines<BufReader<File>>>> = SyncUnsafeCell::new(None);

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
    Other,
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
    let gr_gid = match buf[0..mem::size_of::<gid_t>()].try_into() {
        Ok(buf) => gid_t::from_ne_bytes(buf),
        Err(err) => return None,
    };

    // Get address of buffer for fixing up gr_mem
    let buf_addr = buf.as_ptr() as usize;

    // We moved the gid to the beginning of the byte buffer so we can do this.
    let mut parts = buf[mem::size_of::<gid_t>()..].split_mut(|&c| c == b'\0');
    let gr_name = parts.next()?.as_mut_ptr() as *mut c_char;
    let gr_passwd = parts.next()?.as_mut_ptr() as *mut c_char;
    let gr_mem = parts.next()?.as_mut_ptr() as *mut usize;

    // Adjust gr_mem address by buffer base address
    // TODO: max group members length?
    for i in 0..4096 {
        unsafe {
            if *gr_mem.add(i) == 0 {
                // End of gr_mem pointer array
                break;
            }
            *gr_mem.add(i) += buf_addr;
        }
    }

    Some(group {
        gr_name,
        gr_passwd,
        gr_gid,
        gr_mem: gr_mem as *mut *mut c_char,
    })
}

fn parse_grp(line: String, destbuf: Option<DestBuffer>) -> Result<OwnedGrp, Error> {
    let mut buffer = line.to_owned().into_bytes();

    let mut buffer = buffer
        .into_iter()
        .map(|i| if i == SEPARATOR as u8 { b'\0' } else { i })
        .chain([b'\0'])
        .collect::<Vec<_>>();
    let mut buffer = buffer.split_mut(|i| *i == b'\0');

    let mut gr_gid: gid_t = 0;
    let strings = {
        let mut vec: Vec<u8> = Vec::new();

        let gr_name = buffer.next().ok_or(Error::EOF)?.to_vec();
        let gr_passwd = if cfg!(target_os = "redox") {
            Vec::new()
        } else {
            buffer.next().ok_or(Error::EOF)?.to_vec()
        };
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

        let members = buffer.next().ok_or(Error::EOF)?;

        // Get the pointer to the members array
        let member_array_ptr = unsafe { vec.as_mut_ptr().add(vec.len()) as *mut usize };

        // Push enough null pointers to fit all members
        for _member in members
            .split(|b| *b == b',')
            .filter(|member| !member.is_empty())
        {
            vec.extend(0usize.to_ne_bytes());
        }
        // Push a null pointer to terminate the members array
        vec.extend(0usize.to_ne_bytes());

        // Fill in member names
        for (i, member) in members
            .split(|b| *b == b',')
            .filter(|member| !member.is_empty())
            .enumerate()
        {
            // Store offset to start of member, MUST BE ADJUSTED LATER BASED ON THE ADDRESS OF THE BUFFER
            unsafe {
                *member_array_ptr.add(i) = vec.len();
            }

            vec.extend(member);
            vec.push(0);
        }

        vec
    };

    let mut buffer = match destbuf {
        None => MaybeAllocated::Owned(Box::into_pin(strings.into_boxed_slice())),
        Some(buf) => {
            let mut buf = MaybeAllocated::Borrowed(buf);

            if buf.len() < buf.len() {
                platform::ERRNO.set(errno::ERANGE);
                return Err(Error::BufTooSmall);
            }

            buf[..strings.len()].copy_from_slice(&strings);
            buf
        }
    };
    let reference = split(&mut buffer).ok_or(Error::Other)?;

    Ok(OwnedGrp { buffer, reference })
}

// MT-Unsafe race:grgid locale
#[no_mangle]
pub unsafe extern "C" fn getgrgid(gid: gid_t) -> *mut group {
    let Ok(db) = File::open(c_str!("/etc/group"), fcntl::O_RDONLY) else {
        return ptr::null_mut();
    };

    for line in BufReader::new(db).lines() {
        let Ok(line) = line else {
            return ptr::null_mut();
        };
        let Ok(grp) = parse_grp(line, None) else {
            return ptr::null_mut();
        };

        if grp.reference.gr_gid == gid {
            return grp.into_global();
        }
    }

    return ptr::null_mut();
}

// MT-Unsafe race:grnam locale
#[no_mangle]
pub unsafe extern "C" fn getgrnam(name: *const c_char) -> *mut group {
    let Ok(db) = File::open(c_str!("/etc/group"), fcntl::O_RDONLY) else {
        return ptr::null_mut();
    };

    for line in BufReader::new(db).lines() {
        let Ok(line) = line else {
            return ptr::null_mut();
        };

        let Ok(grp) = parse_grp(line, None) else {
            return ptr::null_mut();
        };

        // Attempt to prevent BO vulnerabilities
        if strncmp(
            grp.reference.gr_name,
            name,
            strlen(grp.reference.gr_name).min(strlen(name)),
        ) > 0
        {
            return grp.into_global();
        }
    }

    return ptr::null_mut();
}

// MT-Safe locale
#[no_mangle]
pub unsafe extern "C" fn getgrgid_r(
    gid: gid_t,
    result_buf: *mut group,
    buffer: *mut c_char,
    buflen: usize,
    result: *mut *mut group,
) -> c_int {
    let Ok(db) = File::open(c_str!("/etc/group"), fcntl::O_RDONLY) else {
        return ENOENT;
    };

    for line in BufReader::new(db).lines() {
        let Ok(line) = line else { return EINVAL };
        let Ok(mut grp) = parse_grp(
            line,
            Some(DestBuffer {
                ptr: buffer as *mut u8,
                len: buflen,
            }),
        ) else {
            return EINVAL;
        };

        if grp.reference.gr_gid == gid {
            *result_buf = grp.reference;
            *result = result_buf;

            return 0;
        }
    }

    return ENOENT;
}

// MT-Safe locale
#[no_mangle]
pub unsafe extern "C" fn getgrnam_r(
    name: *const c_char,
    result_buf: *mut group,
    buffer: *mut c_char,
    buflen: usize,
    result: *mut *mut group,
) -> c_int {
    let Ok(db) = File::open(c_str!("/etc/group"), fcntl::O_RDONLY) else {
        return ENOENT;
    };

    for line in BufReader::new(db).lines() {
        let Ok(line) = line else { return EINVAL };
        let Ok(mut grp) = parse_grp(
            line,
            Some(DestBuffer {
                ptr: buffer as *mut u8,
                len: buflen,
            }),
        ) else {
            return EINVAL;
        };

        if strncmp(
            grp.reference.gr_name,
            name,
            strlen(grp.reference.gr_name).min(strlen(name)),
        ) > 0
        {
            *result_buf = grp.reference;
            *result = result_buf;

            return 0;
        }
    }

    return ENOENT;
}

// MT-Unsafe race:grent race:grentbuf locale
#[no_mangle]
pub unsafe extern "C" fn getgrent() -> *mut group {
    let mut line_reader = &mut *LINE_READER.get();

    if line_reader.is_none() {
        let Ok(db) = File::open(c_str!("/etc/group"), fcntl::O_RDONLY) else {
            return ptr::null_mut();
        };
        *line_reader = Some(BufReader::new(db).lines());
    }

    if let Some(lines) = line_reader.deref_mut() {
        let Some(line) = lines.next() else {
            return ptr::null_mut();
        };
        let Ok(line) = line else {
            return ptr::null_mut();
        };

        if let Ok(grp) = parse_grp(line, None) {
            return grp.into_global();
        } else {
            return ptr::null_mut();
        }
    } else {
        return ptr::null_mut();
    }
}

// MT-Unsafe race:grent locale
#[no_mangle]
pub unsafe extern "C" fn endgrent() {
    *(&mut *LINE_READER.get()) = None;
}

// MT-Unsafe race:grent locale
#[no_mangle]
pub unsafe extern "C" fn setgrent() {
    let mut line_reader = &mut *LINE_READER.get();
    let Ok(db) = File::open(c_str!("/etc/group"), fcntl::O_RDONLY) else {
        return;
    };
    *line_reader = Some(BufReader::new(db).lines());
}

// MT-Safe locale
// Not POSIX
#[no_mangle]
pub unsafe extern "C" fn getgrouplist(
    user: *const c_char,
    group: gid_t,
    groups: *mut gid_t,
    ngroups: *mut c_int,
) -> c_int {
    let mut grps =
        slice::from_raw_parts_mut(groups.cast::<MaybeUninit<gid_t>>(), ngroups.read() as usize);

    // FIXME: This API probably expects the group database to already exist in memory, as it
    // doesn't seem to have any documented error handling.

    let Ok(user) = (crate::c_str::CStr::from_ptr(user).to_str()) else {
        return 0;
    };

    let Ok(db) = File::open(c_str!("/etc/group"), fcntl::O_RDONLY) else {
        return 0;
    };

    let mut groups_found: c_int = 0;

    for line in BufReader::new(db).lines() {
        let Ok(line) = line else {
            return 0;
        };

        let mut parts = line.split(SEPARATOR);

        let group_name = parts.next().unwrap_or("");
        let group_password = parts.next().unwrap_or("");
        let group_id = parts.next().unwrap_or("-1").parse::<c_int>().unwrap();
        let members = parts
            .next()
            .unwrap_or("")
            .split(",")
            .map(|i| i.trim())
            .collect::<Vec<_>>();

        if !members.iter().any(|i| *i == user) {
            continue;
        }

        if let Some(dst) = grps.get_mut(groups_found as usize) {
            dst.write(group_id);
        }

        groups_found = match groups_found.checked_add(1) {
            Some(g) => g,
            None => break,
        };
    }

    ngroups.write(groups_found);

    if groups_found as usize > grps.len() {
        -1
    } else {
        grps.len() as c_int
    }
}

// MT-Safe locale
// Not POSIX
#[no_mangle]
pub unsafe extern "C" fn initgroups(user: *const c_char, gid: gid_t) -> c_int {
    let mut groups = [0; limits::NGROUPS_MAX];
    let mut count = groups.len() as c_int;
    if getgrouplist(user, gid, groups.as_mut_ptr(), &mut count) < 0 {
        return -1;
    }
    unistd::setgroups(count as size_t, groups.as_ptr())
}
