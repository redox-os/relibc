#![allow(non_camel_case_types, dead_code)]

use libc;

pub type pid_t = libc::c_int;

pub type time_t = i64;
pub type suseconds_t = libc::c_long;

// Socket related types
pub type in_addr_t = [u8; 4];
pub type sa_family_t = u16;
pub type socklen_t = libc::size_t;
pub type in_port_t = [u8; 2];

// Statvfs types
pub type dev_t = libc::c_ulong;
pub type ino_t = libc::c_ulong;
pub type mode_t = libc::c_uint;
pub type nlink_t = libc::c_ulong;
pub type uid_t = libc::c_uint;
pub type gid_t = libc::c_uint;
pub type off_t = libc::c_long;
pub type blksize_t = libc::c_long;
pub type blkcnt_t = libc::c_long;
pub type fsblkcnt_t = libc::c_ulong;
pub type fsfilcnt_t = libc::c_ulong;
pub type __fsword_t = libc::c_long;

#[repr(C)]
pub struct fsid_t {
    __val: [libc::c_int; 2]
}

#[repr(C)]
#[derive(Clone,Copy)]
pub struct in_addr {
    pub s_addr: in_addr_t
}

#[repr(C)]
pub struct sockaddr {
    pub sa_family: sa_family_t,
    sa_data: [libc::c_char; 14]
}

#[repr(C)]
pub struct sockaddr_in {
    pub sin_family: sa_family_t,
    pub sin_port: in_port_t,
    pub sin_addr: in_addr,
    __pad: [u8; 8]
}

#[repr(C)]
pub struct hostent {
    pub h_name: *const libc::c_char,
    pub h_aliases: *const *const libc::c_char,
    pub h_addrtype: libc::c_int,
    pub h_length: libc::c_int,
    pub h_addr_list: *const *const libc::c_char
}

#[repr(C)]
#[derive(Debug)]
pub struct timeval {
    pub tv_sec: time_t,
    pub tv_usec: suseconds_t
}

#[repr(C)]
pub struct utimbuf {
    pub actime: time_t,
    pub modtime: time_t
}

pub type fd_mask = libc::c_ulong;
pub const FD_SETSIZE: usize = 64;
pub const NFDBITS: usize = 8 * 8; // Bits in a fd_mask

#[repr(C)]
#[derive(Debug)]
pub struct fd_set {
    pub fds_bits: [fd_mask; (FD_SETSIZE + NFDBITS - 1) / NFDBITS]
}

pub const POLLIN: libc::c_short = 0x0001;
pub const POLLPRI: libc::c_short = 0x0002;
pub const POLLOUT: libc::c_short = 0x0004;
pub const POLLERR: libc::c_short = 0x0008;
pub const POLLHUP: libc::c_short = 0x0010;
pub const POLLNVAL: libc::c_short = 0x0020;

#[repr(C)]
pub struct pollfd {
    pub fd: libc::c_int,
    pub events: libc::c_short,
    pub revents: libc::c_short,
}

#[repr(C)]
pub struct passwd {
    pub pw_name: *const libc::c_char,
    pub pw_passwd: *const libc::c_char,
    pub pw_uid: libc::uid_t,
    pub pw_gid: libc::gid_t,
    pub pw_gecos: *const libc::c_char,
    pub pw_dir: *const libc::c_char,
    pub pw_shell: *const libc::c_char
}
