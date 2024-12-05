//! netdb implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xns/netdb.h.html

mod dns;

use core::{
    cell::Cell,
    fmt::Write,
    mem, ptr, slice,
    str::{self, FromStr},
};

use alloc::{borrow::ToOwned, boxed::Box, str::SplitWhitespace, vec::Vec};

use crate::{
    c_str::{CStr, CString},
    error::ResultExt,
    header::{
        arpa_inet::{htons, inet_aton, ntohl},
        errno::*,
        fcntl::O_RDONLY,
        netinet_in::{in_addr, sockaddr_in, sockaddr_in6},
        stdlib::atoi,
        strings::strcasecmp,
        sys_socket::{constants::AF_INET, sa_family_t, sockaddr, socklen_t},
        unistd::SEEK_SET,
    },
    platform::{
        self,
        rlb::{Line, RawLineBuffer},
        types::*,
        Pal, Sys,
    },
};

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod sys;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;

pub use self::host::*;
pub mod host;

pub use self::lookup::*;
pub mod lookup;

#[repr(C)]
pub struct hostent {
    h_name: *mut c_char,
    h_aliases: *mut *mut c_char,
    h_addrtype: c_int,
    h_length: c_int,
    h_addr_list: *mut *mut c_char,
}

#[repr(C)]
pub struct netent {
    n_name: *mut c_char,         /* official name of net */
    n_aliases: *mut *mut c_char, /* alias list */
    n_addrtype: c_int,           /* net address type */
    n_net: c_ulong,              /* network # */
}

#[repr(C)]
pub struct protoent {
    p_name: *mut c_char,         /* official protocol name */
    p_aliases: *mut *mut c_char, /* alias list */
    p_proto: c_int,              /* protocol # */
}

#[repr(C)]
pub struct servent {
    s_name: *mut c_char,         /* official service name */
    s_aliases: *mut *mut c_char, /* alias list */
    s_port: c_int,               /* port # */
    s_proto: *mut c_char,        /* protocol to use */
}

#[repr(C)]
#[derive(Debug)]
pub struct addrinfo {
    ai_flags: c_int,           /* AI_PASSIVE, AI_CANONNAME, AI_NUMERICHOST */
    ai_family: c_int,          /* PF_xxx */
    ai_socktype: c_int,        /* SOCK_xxx */
    ai_protocol: c_int,        /* 0 or IPPROTO_xxx for IPv4 and IPv6 */
    ai_addrlen: size_t,        /* length of ai_addr */
    ai_canonname: *mut c_char, /* canonical name for hostname */
    ai_addr: *mut sockaddr,    /* binary address */
    ai_next: *mut addrinfo,    /* next structure in linked list */
}

pub const AI_PASSIVE: c_int = 0x0001;
pub const AI_CANONNAME: c_int = 0x0002;
pub const AI_NUMERICHOST: c_int = 0x0004;
pub const AI_V4MAPPED: c_int = 0x0008;
pub const AI_ALL: c_int = 0x0010;
pub const AI_ADDRCONFIG: c_int = 0x0020;
pub const AI_NUMERICSERV: c_int = 0x0400;

pub const EAI_BADFLAGS: c_int = -1;
pub const EAI_NONAME: c_int = -2;
pub const EAI_AGAIN: c_int = -3;
pub const EAI_FAIL: c_int = -4;
pub const EAI_NODATA: c_int = -5;
pub const EAI_FAMILY: c_int = -6;
pub const EAI_SOCKTYPE: c_int = -7;
pub const EAI_SERVICE: c_int = -8;
pub const EAI_ADDRFAMILY: c_int = -9;
pub const EAI_MEMORY: c_int = -10;
pub const EAI_SYSTEM: c_int = -11;
pub const EAI_OVERFLOW: c_int = -12;

pub const NI_MAXHOST: c_int = 1025;
pub const NI_MAXSERV: c_int = 32;

pub const NI_NUMERICHOST: c_int = 0x0001;
pub const NI_NUMERICSERV: c_int = 0x0002;
pub const NI_NOFQDN: c_int = 0x0004;
pub const NI_NAMEREQD: c_int = 0x0008;
pub const NI_DGRAM: c_int = 0x0010;

static mut NETDB: c_int = 0;
pub static mut NET_ENTRY: netent = netent {
    n_name: ptr::null_mut(),
    n_aliases: ptr::null_mut(),
    n_addrtype: 0,
    n_net: 0,
};
pub static mut NET_NAME: Option<Vec<u8>> = None;
pub static mut NET_ALIASES: Option<Vec<Vec<u8>>> = None;
pub static mut NET_ADDR: Option<u32> = None;
static mut N_POS: usize = 0;
static mut NET_STAYOPEN: c_int = 0;

#[thread_local]
pub static H_ERRNO: Cell<c_int> = Cell::new(0);
const H_UNSET: c_int = 0;
pub const HOST_NOT_FOUND: c_int = 1;
pub const NO_DATA: c_int = 2;
pub const NO_RECOVERY: c_int = 3;
pub const TRY_AGAIN: c_int = 4;

// Expected length of addresses
const SOCKLEN_AF_INET4: socklen_t = 4;
const SOCKLEN_AF_INET6: socklen_t = 16;

static mut PROTODB: c_int = 0;
static mut PROTO_ENTRY: protoent = protoent {
    p_name: ptr::null_mut(),
    p_aliases: ptr::null_mut(),
    p_proto: 0 as c_int,
};
static mut PROTO_NAME: Option<Vec<u8>> = None;
static mut PROTO_ALIASES: Option<Vec<Vec<u8>>> = None;
static mut PROTO_NUM: Option<c_int> = None;
static mut P_POS: usize = 0;
static mut PROTO_STAYOPEN: c_int = 0;

static mut SERVDB: c_int = 0;
static mut SERV_ENTRY: servent = servent {
    s_name: ptr::null_mut(),
    s_aliases: ptr::null_mut(),
    s_port: 0 as c_int,
    s_proto: ptr::null_mut(),
};
static mut SERV_NAME: Option<Vec<u8>> = None;
static mut SERV_ALIASES: Option<Vec<Vec<u8>>> = None;
static mut SERV_PORT: Option<c_int> = None;
static mut SERV_PROTO: Option<Vec<u8>> = None;
static mut S_POS: usize = 0;
static mut SERV_STAYOPEN: c_int = 0;

fn bytes_to_box_str(bytes: &[u8]) -> Box<str> {
    Box::from(core::str::from_utf8(bytes).unwrap_or(""))
}

#[no_mangle]
pub unsafe extern "C" fn endnetent() {
    Sys::close(NETDB);
    NETDB = 0;
}

#[no_mangle]
pub unsafe extern "C" fn endprotoent() {
    Sys::close(PROTODB);
    PROTODB = 0;
}

#[no_mangle]
pub unsafe extern "C" fn endservent() {
    Sys::close(SERVDB);
    SERVDB = 0;
}

/// Resolve a host name from a given network address.
///
/// # Arguments
/// * `v` - Address to resolve as a non-null [`in_addr`]
/// * `length` -
/// * `format` - AF_INET or AF_INET6
///
/// # Safety
/// * `v` must be a valid pointer.
/// * `length` must correctly match the size of `v` as expected by `format` (usually 4 or 16).
/// * This function is not reentrant and may modify static data.
///
/// # Panics
/// Panics if `v` is a null pointer.
///
/// # Deprecation
/// Deprecated as of POSIX.1-2001 and removed in POSIX.1-2008.
/// New code should use [`getaddrinfo`] instead.
#[no_mangle]
#[deprecated]
pub unsafe extern "C" fn gethostbyaddr(
    v: *const c_void,
    length: socklen_t,
    format: c_int,
) -> *mut hostent {
    assert!(
        !v.is_null(),
        "`gethostbyaddr()` called with null `v` (in_addr)"
    );
    // Uncomment if optional IPv6 support is added
    // if length != SOCKLEN_AF_INET4 || length != SOCKLEN_AF_INET6 {
    //     H_ERRNO.set(NO_RECOVERY);
    //     return ptr::null_mut();
    // }
    if length != SOCKLEN_AF_INET4 {
        H_ERRNO.set(NO_RECOVERY);
        return ptr::null_mut();
    }
    let addr: in_addr = *(v as *mut in_addr);

    // check the hosts file first
    let mut p: *mut hostent;
    sethostent(HOST_STAYOPEN);
    while {
        p = gethostent();
        !p.is_null()
    } {
        let mut cp = (*p).h_addr_list;
        loop {
            if cp.is_null() {
                break;
            }
            if (*cp).is_null() {
                break;
            }
            let mut cp_slice: [i8; 4] = [0i8; 4];
            (*cp).copy_to(cp_slice.as_mut_ptr(), 4);
            let cp_s_addr = mem::transmute::<[i8; 4], u32>(cp_slice);
            if cp_s_addr == addr.s_addr {
                sethostent(HOST_STAYOPEN);
                return p;
            }
            cp = cp.offset(1);
        }
    }

    //TODO actually get aliases
    let mut _host_aliases: Vec<Vec<u8>> = Vec::new();
    _host_aliases.push(vec![b'\0']);
    let mut host_aliases: Vec<*mut i8> = Vec::new();
    host_aliases.push(ptr::null_mut());
    HOST_ALIASES = Some(_host_aliases);

    match lookup_addr(addr).map(|host_names| host_names.into_iter().next()) {
        Ok(Some(host_name)) => {
            _HOST_ADDR_LIST = addr.s_addr.to_ne_bytes();
            HOST_ADDR_LIST = [_HOST_ADDR_LIST.as_mut_ptr() as *mut c_char, ptr::null_mut()];
            HOST_NAME = Some(host_name);
            HOST_ENTRY = hostent {
                h_name: HOST_NAME.as_mut().unwrap().as_mut_ptr() as *mut c_char,
                h_aliases: host_aliases.as_mut_slice().as_mut_ptr() as *mut *mut i8,
                h_addrtype: format,
                h_length: length as i32,
                h_addr_list: HOST_ADDR_LIST.as_mut_ptr(),
            };
            &mut HOST_ENTRY
        }
        // `glibc` sets errno if an address doesn't have a host name
        // `musl` uses the address as the host name in said case
        Ok(None) => {
            H_ERRNO.set(HOST_NOT_FOUND);
            ptr::null_mut()
        }
        Err(e) => {
            // TODO: Better error separation in lookup_addr
            H_ERRNO.set(NO_RECOVERY);
            ptr::null_mut()
        }
    }
}

/// Resolve host information by name or IP address.
///
/// # Arguments
/// * `name` - Host name or IP address.
///
/// # Safety
/// `name` must be a valid string.
/// This function is not reentrant and may modify static data.
///
/// # Panics
/// Panics if `name` is a null pointer.
///
/// # Deprecation
/// Deprecated as of POSIX.1-2001 and removed in POSIX.1-2008.
/// New code should use [`getaddrinfo`] instead.
#[no_mangle]
#[deprecated]
pub unsafe extern "C" fn gethostbyname(name: *const c_char) -> *mut hostent {
    let name_cstr =
        CStr::from_nullable_ptr(name).expect("gethostbyname() called with a NULL pointer");
    let Ok(name_str) = str::from_utf8(name_cstr.to_bytes()) else {
        H_ERRNO.set(NO_RECOVERY);
        return ptr::null_mut();
    };

    // Addresses and hostnames are both valid, so we'll check addresses first
    // The standard doesn't define what to do when called with addresses
    // Some implementations just skip resolution and copy the address to h_name
    let mut octets = name_str.split('.');
    let mut s_addr = [0u8; 4];
    let mut is_addr = true;
    for item in &mut s_addr {
        if let Some(n) = octets.next().and_then(|x| u8::from_str(x).ok()) {
            *item = n;
        } else {
            is_addr = false;
            break;
        }
    }
    if octets.next() != None {
        is_addr = false;
    }

    if is_addr {
        let s_addr = u32::from_ne_bytes(s_addr);
        let addr = in_addr { s_addr };
        return gethostbyaddr(&addr as *const _ as *const c_void, 4, AF_INET);
    }

    // check the hosts file first
    let mut p: *mut hostent;
    sethostent(HOST_STAYOPEN);
    while {
        p = gethostent();
        !p.is_null()
    } {
        if strcasecmp((*p).h_name, name) == 0 {
            sethostent(HOST_STAYOPEN);
            return p;
        }
        let mut cp = (*p).h_aliases;
        loop {
            if cp.is_null() {
                break;
            }
            if (*cp).is_null() {
                break;
            }
            if strcasecmp(*cp, name) == 0 {
                sethostent(HOST_STAYOPEN);
                return p;
            }
            cp = cp.offset(1);
        }
    }

    let mut host = match lookup_host(name_str) {
        Ok(lookuphost) => lookuphost,
        Err(e) => {
            H_ERRNO.set(NO_RECOVERY);
            return ptr::null_mut();
        }
    };
    let host_addr = match host.next() {
        Some(result) => result,
        None => {
            H_ERRNO.set(HOST_NOT_FOUND);
            return ptr::null_mut();
        }
    };

    let host_name: Vec<u8> = name_cstr.to_bytes().to_vec();
    HOST_NAME = Some(host_name);
    _HOST_ADDR_LIST = host_addr.s_addr.to_ne_bytes();
    HOST_ADDR_LIST = [_HOST_ADDR_LIST.as_mut_ptr() as *mut c_char, ptr::null_mut()];
    HOST_ADDR = Some(host_addr);

    //TODO actually get aliases
    let mut _host_aliases: Vec<Vec<u8>> = Vec::new();
    _host_aliases.push(vec![b'\0']);
    let mut host_aliases: Vec<*mut i8> = Vec::new();
    host_aliases.push(ptr::null_mut());
    host_aliases.push(ptr::null_mut());
    HOST_ALIASES = Some(_host_aliases);

    HOST_ENTRY = hostent {
        h_name: HOST_NAME.as_mut().unwrap().as_mut_ptr() as *mut c_char,
        h_aliases: host_aliases.as_mut_slice().as_mut_ptr() as *mut *mut i8,
        h_addrtype: AF_INET,
        h_length: 4,
        h_addr_list: HOST_ADDR_LIST.as_mut_ptr(),
    };
    sethostent(HOST_STAYOPEN);
    &mut HOST_ENTRY as *mut hostent
}

pub unsafe extern "C" fn getnetbyaddr(net: u32, net_type: c_int) -> *mut netent {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn getnetbyname(name: *const c_char) -> *mut netent {
    let mut n: *mut netent;
    setnetent(NET_STAYOPEN);
    while {
        n = getnetent();
        !n.is_null()
    } {
        if strcasecmp((*n).n_name, name) == 0 {
            setnetent(NET_STAYOPEN);
            return n;
        }
    }
    setnetent(NET_STAYOPEN);

    platform::ERRNO.set(ENOENT);
    ptr::null_mut() as *mut netent
}

#[no_mangle]
pub unsafe extern "C" fn getnetent() -> *mut netent {
    // TODO: Rustify implementation

    if NETDB == 0 {
        NETDB = Sys::open(c_str!("/etc/networks"), O_RDONLY, 0).or_minus_one_errno();
    }

    let mut rlb = RawLineBuffer::new(NETDB);
    rlb.seek(N_POS);

    let mut r: Box<str> = Box::default();
    while r.is_empty() || r.split_whitespace().next() == None || r.starts_with('#') {
        r = match rlb.next() {
            Line::Some(s) => bytes_to_box_str(s),
            _ => {
                if NET_STAYOPEN == 0 {
                    endnetent();
                }
                return ptr::null_mut();
            }
        };
    }
    rlb.next();
    N_POS = rlb.line_pos();

    let mut iter: SplitWhitespace = r.split_whitespace();

    let net_name = iter.next().unwrap().bytes().chain(Some(b'\0')).collect();
    NET_NAME = Some(net_name);

    let addr_vec: Vec<u8> = iter.next().unwrap().bytes().chain(Some(b'\0')).collect();
    let addr_cstr = addr_vec.as_slice().as_ptr() as *const i8;
    let mut addr = mem::MaybeUninit::uninit();
    inet_aton(addr_cstr, addr.as_mut_ptr());
    let addr = addr.assume_init();
    NET_ADDR = Some(ntohl(addr.s_addr));

    let mut _net_aliases: Vec<Vec<u8>> = iter
        .map(|alias| alias.bytes().chain(Some(b'\0')).collect())
        .collect();
    let mut net_aliases: Vec<*mut i8> = _net_aliases
        .iter_mut()
        .map(|x| x.as_mut_ptr() as *mut i8)
        .chain(Some(ptr::null_mut()))
        .collect();
    NET_ALIASES = Some(_net_aliases);

    NET_ENTRY = netent {
        n_name: NET_NAME.as_mut().unwrap().as_mut_ptr() as *mut c_char,
        n_aliases: net_aliases.as_mut_slice().as_mut_ptr() as *mut *mut i8,
        n_addrtype: AF_INET,
        n_net: NET_ADDR.unwrap() as c_ulong,
    };
    &mut NET_ENTRY as *mut netent
}

#[no_mangle]
pub unsafe extern "C" fn getprotobyname(name: *const c_char) -> *mut protoent {
    let mut p: *mut protoent;
    setprotoent(PROTO_STAYOPEN);
    while {
        p = getprotoent();
        !p.is_null()
    } {
        if strcasecmp((*p).p_name, name) == 0 {
            setprotoent(PROTO_STAYOPEN);
            return p;
        }

        let mut cp = (*p).p_aliases;
        loop {
            if cp.is_null() {
                setprotoent(PROTO_STAYOPEN);
                break;
            }
            if (*cp).is_null() {
                setprotoent(PROTO_STAYOPEN);
                break;
            }
            if strcasecmp(*cp, name) == 0 {
                setprotoent(PROTO_STAYOPEN);
                return p;
            }
            cp = cp.offset(1);
        }
    }
    setprotoent(PROTO_STAYOPEN);

    platform::ERRNO.set(ENOENT);
    ptr::null_mut() as *mut protoent
}

#[no_mangle]
pub unsafe extern "C" fn getprotobynumber(number: c_int) -> *mut protoent {
    setprotoent(PROTO_STAYOPEN);
    let mut p: *mut protoent;
    while {
        p = getprotoent();
        !p.is_null()
    } {
        if (*p).p_proto == number {
            setprotoent(PROTO_STAYOPEN);
            return p;
        }
    }
    setprotoent(PROTO_STAYOPEN);
    platform::ERRNO.set(ENOENT);
    ptr::null_mut() as *mut protoent
}

#[no_mangle]
pub unsafe extern "C" fn getprotoent() -> *mut protoent {
    if PROTODB == 0 {
        PROTODB = Sys::open(c_str!("/etc/protocols"), O_RDONLY, 0).or_minus_one_errno();
    }

    let mut rlb = RawLineBuffer::new(PROTODB);
    rlb.seek(P_POS);

    let mut r: Box<str> = Box::default();
    while r.is_empty() || r.split_whitespace().next() == None || r.starts_with('#') {
        r = match rlb.next() {
            Line::Some(s) => bytes_to_box_str(s),
            _ => {
                if PROTO_STAYOPEN == 0 {
                    endprotoent();
                }
                return ptr::null_mut();
            }
        };
    }
    rlb.next();
    P_POS = rlb.line_pos();

    let mut iter: SplitWhitespace = r.split_whitespace();

    let mut proto_name: Vec<u8> = iter.next().unwrap().as_bytes().to_vec();
    proto_name.push(b'\0');

    let mut num = iter.next().unwrap().as_bytes().to_vec();
    num.push(b'\0');
    PROTO_NUM = Some(atoi(num.as_mut_slice().as_mut_ptr() as *mut i8));

    let mut _proto_aliases: Vec<Vec<u8>> = iter
        .map(|alias| alias.bytes().chain(Some(b'\0')).collect())
        .collect();
    let mut proto_aliases: Vec<*mut i8> = _proto_aliases
        .iter_mut()
        .map(|x| x.as_mut_ptr() as *mut i8)
        .chain(Some(ptr::null_mut()))
        .collect();

    PROTO_ALIASES = Some(_proto_aliases);
    PROTO_NAME = Some(proto_name);

    PROTO_ENTRY = protoent {
        p_name: PROTO_NAME.as_mut().unwrap().as_mut_slice().as_mut_ptr() as *mut c_char,
        p_aliases: proto_aliases.as_mut_slice().as_mut_ptr() as *mut *mut i8,
        p_proto: PROTO_NUM.unwrap(),
    };
    if PROTO_STAYOPEN == 0 {
        endprotoent();
    }
    &mut PROTO_ENTRY as *mut protoent
}

#[no_mangle]
pub unsafe extern "C" fn getservbyname(name: *const c_char, proto: *const c_char) -> *mut servent {
    setservent(SERV_STAYOPEN);
    let mut p: *mut servent;
    if proto.is_null() {
        while {
            p = getservent();
            !p.is_null()
        } {
            if strcasecmp((*p).s_name, name) == 0 {
                setservent(SERV_STAYOPEN);
                return p;
            }
        }
    } else {
        while {
            p = getservent();
            !p.is_null()
        } {
            if strcasecmp((*p).s_name, name) == 0 && strcasecmp((*p).s_proto, proto) == 0 {
                setservent(SERV_STAYOPEN);
                return p;
            }
        }
    }
    setservent(SERV_STAYOPEN);
    platform::ERRNO.set(ENOENT);
    ptr::null_mut() as *mut servent
}

#[no_mangle]
pub unsafe extern "C" fn getservbyport(port: c_int, proto: *const c_char) -> *mut servent {
    setservent(SERV_STAYOPEN);
    let mut p: *mut servent;
    if proto.is_null() {
        while {
            p = getservent();
            !p.is_null()
        } {
            if (*p).s_port == port {
                setservent(SERV_STAYOPEN);
                return p;
            }
        }
    } else {
        while {
            p = getservent();
            !p.is_null()
        } {
            if (*p).s_port == port && strcasecmp((*p).s_proto, proto) == 0 {
                setservent(SERV_STAYOPEN);
                return p;
            }
        }
    }
    setservent(SERV_STAYOPEN);
    platform::ERRNO.set(ENOENT);
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn getservent() -> *mut servent {
    if SERVDB == 0 {
        // TODO: Rustify
        SERVDB = Sys::open(c_str!("/etc/services"), O_RDONLY, 0).or_minus_one_errno();
    }
    let mut rlb = RawLineBuffer::new(SERVDB);
    rlb.seek(S_POS);

    let r: Box<str> = Box::default();

    loop {
        let r = match rlb.next() {
            Line::Some(s) => bytes_to_box_str(s),
            _ => {
                if SERV_STAYOPEN == 0 {
                    endservent();
                }
                return ptr::null_mut();
            }
        };

        let mut iter = r.split_whitespace();
        let serv_name = match iter.next() {
            Some(serv_name) => serv_name.bytes().chain(Some(b'\0')).collect(),
            None => continue,
        };
        let port_proto = match iter.next() {
            Some(port_proto) => port_proto,
            None => continue,
        };
        let mut split = port_proto.split('/');
        let mut port: Vec<u8> = match split.next() {
            Some(port) => port.bytes().chain(Some(b'\0')).collect(),
            None => continue,
        };
        SERV_PORT =
            Some(htons(atoi(port.as_mut_slice().as_mut_ptr() as *mut i8) as u16) as u32 as i32);
        let proto = match split.next() {
            Some(proto) => proto.bytes().chain(Some(b'\0')).collect(),
            None => continue,
        };

        rlb.next();
        S_POS = rlb.line_pos();

        /*
         *let mut _serv_aliases: Vec<Vec<u8>> = Vec::new();
         *loop {
         *    let mut alias = match iter.next() {
         *        Some(s) => s.as_bytes().to_vec(),
         *        _ => break
         *    };
         *    alias.push(b'\0');
         *    _serv_aliases.push(alias);
         *}
         *let mut serv_aliases: Vec<*mut i8> = _serv_aliases.iter_mut().map(|x| x.as_mut_ptr() as *mut i8).collect();
         *serv_aliases.push(ptr::null_mut());
         *
         */
        let mut _serv_aliases: Vec<Vec<u8>> = Vec::new();
        _serv_aliases.push(vec![b'\0']);
        let mut serv_aliases: Vec<*mut i8> = Vec::new();
        serv_aliases.push(ptr::null_mut());
        serv_aliases.push(ptr::null_mut());

        SERV_ALIASES = Some(_serv_aliases);
        SERV_NAME = Some(serv_name);
        SERV_PROTO = Some(proto);

        SERV_ENTRY = servent {
            s_name: SERV_NAME.as_mut().unwrap().as_mut_slice().as_mut_ptr() as *mut c_char,
            s_aliases: serv_aliases.as_mut_slice().as_mut_ptr() as *mut *mut i8,
            s_port: SERV_PORT.unwrap(),
            s_proto: SERV_PROTO.as_mut().unwrap().as_mut_slice().as_mut_ptr() as *mut c_char,
        };

        if SERV_STAYOPEN == 0 {
            endservent();
        }
        break &mut SERV_ENTRY as *mut servent;
    }
}

#[no_mangle]
pub unsafe extern "C" fn setnetent(stayopen: c_int) {
    NET_STAYOPEN = stayopen;
    if NETDB == 0 {
        NETDB = Sys::open(c_str!("/etc/networks"), O_RDONLY, 0).or_minus_one_errno()
    } else {
        Sys::lseek(NETDB, 0, SEEK_SET);
        N_POS = 0;
    }
}

#[no_mangle]
pub unsafe extern "C" fn setprotoent(stayopen: c_int) {
    PROTO_STAYOPEN = stayopen;
    if PROTODB == 0 {
        PROTODB = Sys::open(c_str!("/etc/protocols"), O_RDONLY, 0).or_minus_one_errno()
    } else {
        Sys::lseek(PROTODB, 0, SEEK_SET);
        P_POS = 0;
    }
}

#[no_mangle]
pub unsafe extern "C" fn setservent(stayopen: c_int) {
    SERV_STAYOPEN = stayopen;
    if SERVDB == 0 {
        SERVDB = Sys::open(c_str!("/etc/services"), O_RDONLY, 0).or_minus_one_errno()
    } else {
        Sys::lseek(SERVDB, 0, SEEK_SET);
        S_POS = 0;
    }
}

#[no_mangle]
pub unsafe extern "C" fn getaddrinfo(
    node: *const c_char,
    service: *const c_char,
    hints: *const addrinfo,
    res: *mut *mut addrinfo,
) -> c_int {
    //TODO: getaddrinfo
    let node_opt = CStr::from_nullable_ptr(node);
    let service_opt = CStr::from_nullable_ptr(service);

    let hints_opt = if hints.is_null() { None } else { Some(&*hints) };

    trace!(
        "getaddrinfo({:?}, {:?}, {:?})",
        node_opt.map(|c| str::from_utf8_unchecked(c.to_bytes())),
        service_opt.map(|c| str::from_utf8_unchecked(c.to_bytes())),
        hints_opt
    );

    //TODO: Use hints
    let mut ai_flags = hints_opt.map_or(0, |hints| hints.ai_flags);
    let mut ai_family; // = hints_opt.map_or(AF_UNSPEC, |hints| hints.ai_family);
    let ai_socktype = hints_opt.map_or(0, |hints| hints.ai_socktype);
    let mut ai_protocol; // = hints_opt.map_or(0, |hints| hints.ai_protocol);

    *res = ptr::null_mut();

    let mut port = 0;
    if let Some(service) = service_opt {
        //TODO: Support other service definitions as well as AI_NUMERICSERV
        match str::from_utf8_unchecked(service.to_bytes()).parse::<u16>() {
            Ok(ok) => port = ok,
            Err(_err) => (),
        }
    }

    //TODO: Check hosts file
    if let Some(node) = node_opt {
        //TODO: Support AI_NUMERICHOST
        let lookuphost = match lookup_host(str::from_utf8_unchecked(node.to_bytes())) {
            Ok(lookuphost) => lookuphost,
            Err(e) => {
                platform::ERRNO.set(e);
                return EAI_SYSTEM;
            }
        };

        for in_addr in lookuphost {
            ai_family = AF_INET;
            ai_protocol = 0;

            let ai_addr = Box::into_raw(Box::new(sockaddr_in {
                sin_family: ai_family as sa_family_t,
                sin_port: htons(port),
                sin_addr: in_addr,
                sin_zero: [0; 8],
            })) as *mut sockaddr;

            let ai_addrlen = mem::size_of::<sockaddr_in>();

            let ai_canonname = if ai_flags & AI_CANONNAME > 0 {
                ai_flags &= !AI_CANONNAME;
                node.to_owned_cstring().into_raw()
            } else {
                ptr::null_mut()
            };

            let addrinfo = Box::new(addrinfo {
                ai_flags: 0,
                ai_family,
                ai_socktype,
                ai_protocol,
                ai_addrlen,
                ai_canonname,
                ai_addr,
                ai_next: ptr::null_mut(),
            });

            let mut indirect = res;
            while !(*indirect).is_null() {
                indirect = &mut (**indirect).ai_next;
            }
            *indirect = Box::into_raw(addrinfo);
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn getnameinfo(
    addr: *const sockaddr,
    addrlen: socklen_t,
    host: *mut c_char,
    hostlen: socklen_t,
    serv: *mut c_char,
    servlen: socklen_t,
    flags: c_int,
) -> c_int {
    //TODO: getnameinfo
    if addrlen as usize != mem::size_of::<sockaddr_in>() {
        return EAI_FAMILY;
    }

    let addr = &*(addr as *const sockaddr_in);

    let host_opt = if host.is_null() {
        None
    } else {
        Some(slice::from_raw_parts_mut(host, hostlen as usize))
    };

    let serv_opt = if serv.is_null() {
        None
    } else {
        Some(slice::from_raw_parts_mut(serv, servlen as usize))
    };

    eprintln!("getnameinfo({:p}, {}, {:#x})", addr, addrlen, flags);

    platform::ERRNO.set(ENOSYS);
    EAI_SYSTEM
}

#[no_mangle]
pub unsafe extern "C" fn freeaddrinfo(res: *mut addrinfo) {
    let mut ai = res;
    while !ai.is_null() {
        let bai = Box::from_raw(ai);
        if !bai.ai_canonname.is_null() {
            drop(CString::from_raw(bai.ai_canonname));
        }
        if !bai.ai_addr.is_null() {
            if bai.ai_addrlen == mem::size_of::<sockaddr_in>() {
                Box::from_raw(bai.ai_addr as *mut sockaddr_in);
            } else if bai.ai_addrlen == mem::size_of::<sockaddr_in6>() {
                Box::from_raw(bai.ai_addr as *mut sockaddr_in6);
            } else {
                eprintln!("freeaddrinfo: unknown ai_addrlen {}", bai.ai_addrlen);
            }
        }
        ai = bai.ai_next;
    }
}

#[no_mangle]
pub extern "C" fn gai_strerror(errcode: c_int) -> *const c_char {
    match errcode {
        EAI_BADFLAGS => c_str!("Invalid flags"),
        EAI_NONAME => c_str!("Name does not resolve"),
        EAI_AGAIN => c_str!("Try again"),
        EAI_FAIL => c_str!("Non-recoverable error"),
        EAI_NODATA => c_str!("Unknown error"),
        EAI_FAMILY => c_str!("Unrecognized address family or invalid length"),
        EAI_SOCKTYPE => c_str!("Unrecognized socket type"),
        EAI_SERVICE => c_str!("Unrecognized service"),
        EAI_ADDRFAMILY => c_str!("Address family for name not supported"),
        EAI_MEMORY => c_str!("Out of memory"),
        EAI_SYSTEM => c_str!("System error"),
        EAI_OVERFLOW => c_str!("Overflow"),
        _ => c_str!("Unknown error"),
    }
    .as_ptr()
}

/// Provide a pointer to relibc's internal [`H_ERRNO`].
#[no_mangle]
#[deprecated]
pub extern "C" fn __h_errno_location() -> *mut c_int {
    H_ERRNO.as_ptr()
}

#[no_mangle]
#[deprecated]
pub extern "C" fn hstrerror(errcode: c_int) -> *const c_char {
    match errcode {
        H_UNSET => c_str!("Resolver error unset"),
        HOST_NOT_FOUND => c_str!("Unknown hostname"),
        NO_DATA => c_str!("No address for hostname"),
        NO_RECOVERY => c_str!("Unknown server error"),
        TRY_AGAIN => c_str!("Hostname lookup failure"),
        _ => c_str!("Unknown error"),
    }
    .as_ptr()
}

/// Print error message associated with [`H_ERRNO`] to stderr.
///
/// # Arguments
/// * `prefix` - An optional prefix to prepend to the error message. May be null or an empty
/// (`""`) C string.
///
/// # Safety
/// Like [`crate::header::stdio::perror`], `prefix` should be a valid, NUL terminated C string if
/// used. The caller may safely call this function with a null pointer.
///
/// # Deprecation
/// [`H_ERRNO`], [`hstrerror`], [`herror`], and other functions are deprecated as of
/// POSIX.1-2001 and removed as of POSIX.1-2008. These functions are provided for backwards
/// compatibility but should not be used by new code.
#[no_mangle]
#[deprecated]
pub extern "C" fn herror(prefix: *const c_char) {
    let code = H_ERRNO.get();
    // Safety: `hstrerror` handles every error code case and always returns a valid C string
    let error = unsafe {
        let msg_cstr = CStr::from_ptr(hstrerror(code));
        str::from_utf8_unchecked(msg_cstr.to_bytes())
    };

    let mut writer = platform::FileWriter::new(2);
    // Prefix is optional
    match unsafe { CStr::from_nullable_ptr(prefix) }
        .and_then(|prefix| str::from_utf8(prefix.to_bytes()).ok())
    {
        Some(prefix) if !prefix.is_empty() => writer
            .write_fmt(format_args!("{prefix}: {error}\n"))
            .unwrap(),
        _ => writer.write_fmt(format_args!("{error}\n")).unwrap(),
    }
}
