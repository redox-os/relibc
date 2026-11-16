//! `netdb.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netdb.h.html>.

mod dns;

use core::{cell::Cell, fmt::Write, mem, net::Ipv4Addr, ptr, str};

use alloc::{boxed::Box, string::ToString};

use crate::{
    c_str::{CStr, OwnedThinCStr},
    error::Errno,
    header::{
        bits_arpainet::htons,
        bits_safamily_t::sa_family_t,
        errno::{ENOENT, ETIMEDOUT},
        netinet_in::{in_addr, sockaddr_in},
        sys_socket::{
            constants::{AF_INET, AF_INET6, AF_UNSPEC},
            sockaddr, socklen_t,
        },
    },
    platform::types::{c_char, c_int, c_void, uint32_t},
};

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod sys;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;

pub use self::host::*;
pub mod host;

pub use self::net::*;
pub mod net;

pub use self::lookup::*;
pub mod lookup;

pub use self::proto::*;
pub mod proto;

pub use self::serv::*;
pub mod serv;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netdb.h.html>.
#[repr(C)]
pub struct hostent {
    h_name: *mut c_char,
    h_aliases: *mut *mut c_char,
    h_addrtype: c_int,
    h_length: c_int,
    h_addr_list: *mut *mut c_char,
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netdb.h.html>.
#[repr(C)]
#[derive(Default)]
pub struct netent {
    n_name: *mut c_char,         /* official name of net */
    n_aliases: *mut *mut c_char, /* alias list */
    n_addrtype: c_int,           /* net address type */
    n_net: uint32_t,             /* network # */
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netdb.h.html>.
#[repr(C)]
pub struct protoent {
    p_name: *mut c_char,         /* official protocol name */
    p_aliases: *mut *mut c_char, /* alias list */
    p_proto: c_int,              /* protocol # */
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netdb.h.html>.
#[repr(C)]
pub struct servent {
    s_name: *mut c_char,         /* official service name */
    s_aliases: *mut *mut c_char, /* alias list */
    s_port: c_int,               /* port # */
    s_proto: *mut c_char,        /* protocol to use */
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/netdb.h.html>.
#[repr(C)]
#[derive(Debug)]
pub struct addrinfo {
    // NOTE: the soundness assumptions we make here is the fields here can only be accessed by safe
    // code through conversion to/from the wrapped type
    ai_flags: c_int,           /* AI_PASSIVE, AI_CANONNAME, AI_NUMERICHOST */
    ai_family: c_int,          /* PF_xxx */
    ai_socktype: c_int,        /* SOCK_xxx */
    ai_protocol: c_int,        /* 0 or IPPROTO_xxx for IPv4 and IPv6 */
    ai_addrlen: socklen_t,     /* length of ai_addr */
    ai_canonname: *mut c_char, /* canonical name for hostname */
    ai_addr: *mut sockaddr,    /* binary address */
    ai_next: *mut addrinfo,    /* next structure in linked list */
}
struct WrappedAddrInfo {
    flags: c_int,
    family: c_int,
    socktype: c_int,
    protocol: c_int,
    sockaddr: Box<[u8]>,
    canonname: Option<OwnedThinCStr>,
}
impl addrinfo {
    fn from_wrapped(wrapped: WrappedAddrInfo) -> Self {
        Self {
            ai_flags: wrapped.flags,
            ai_family: wrapped.family,
            ai_socktype: wrapped.socktype,
            ai_protocol: wrapped.protocol,
            ai_addrlen: wrapped.sockaddr.len().try_into().unwrap(),
            ai_canonname: wrapped
                .canonname
                .map_or_else(core::ptr::null_mut, OwnedThinCStr::into_ptr),
            ai_addr: Box::into_raw(wrapped.sockaddr).as_mut_ptr().cast(),
            ai_next: core::ptr::null_mut(),
        }
    }
    fn into_wrapped(self) -> (WrappedAddrInfo, Option<Box<Self>>) {
        let next = if self.ai_next.is_null() {
            None
        } else {
            Some(unsafe { Box::from_raw(self.ai_next) })
        };
        let wrapped = WrappedAddrInfo {
            flags: self.ai_flags,
            family: self.ai_family,
            socktype: self.ai_socktype,
            protocol: self.ai_protocol,
            sockaddr: unsafe {
                Box::from_raw(core::ptr::slice_from_raw_parts_mut(
                    self.ai_addr.cast::<u8>(),
                    self.ai_addrlen.try_into().unwrap(),
                ))
            },
            canonname: unsafe { OwnedThinCStr::from_nullable_ptr(self.ai_canonname.cast()) },
        };
        (wrapped, next)
    }
}

/// Socket address is inteded for `bind()`.
pub const AI_PASSIVE: c_int = 0x0001;
/// Request for cannonical name.
pub const AI_CANONNAME: c_int = 0x0002;
/// Return numeric host address as name.
pub const AI_NUMERICHOST: c_int = 0x0004;
/// If no IPv6 addresses are found, query for IPv4 addresses and return them to
/// the caller as IPv4-mapped IPv6 addresses.
pub const AI_V4MAPPED: c_int = 0x0008;
/// Query for both IPv4 and IPv6 addresses.
pub const AI_ALL: c_int = 0x0010;
/// Query for IPv4 addresses only when an IPv4 address is configured; query for
/// IPv6 addresses only when an IPv6 address is configured.
pub const AI_ADDRCONFIG: c_int = 0x0020;
/// Inhibit service name resolution.
pub const AI_NUMERICSERV: c_int = 0x0400;

#[repr(i32)]
enum AddrInfoErr {
    EAI_BADFLAGS = -1,
    EAI_NONAME = -2,
    EAI_AGAIN = -3,
    EAI_FAIL = -4,
    EAI_NODATA = -5,
    EAI_FAMILY = -6,
    EAI_SOCKTYPE = -7,
    EAI_SERVICE = -8,
    EAI_ADDRFAMILY = -9,
    EAI_MEMORY = -10,
    EAI_SYSTEM = -11,
    EAI_OVERFLOW = -12,
}
impl From<Errno> for AddrInfoErr {
    fn from(value: Errno) -> Self {
        value.sync();
        Self::EAI_SYSTEM
    }
}

/// The flags had an invalid value.
pub const EAI_BADFLAGS: c_int = -1;
/// The name does not resolve for the supplied parameters.
/// `NI_NAMEREQD` is set and the host's name cannot be located, or both
/// `nodename` and `servname` were null.
pub const EAI_NONAME: c_int = -2;
/// The name could not be resolved at this time. Future attempts may succeed.
pub const EAI_AGAIN: c_int = -3;
/// A non-recoverable error occurred.
pub const EAI_FAIL: c_int = -4;
// TODO should be guarded by _GNU_SOURCE
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/getaddrinfo.3.html>.
/// The specified network host exists but does not have any network addresses
/// defined.
pub const EAI_NODATA: c_int = -5;
/// The address family was not recognized or the address length was invalid for
/// the specified family.
pub const EAI_FAMILY: c_int = -6;
/// The intended socket type was not recognized.
pub const EAI_SOCKTYPE: c_int = -7;
/// The service passed was not recognized for the specified socket type.
pub const EAI_SERVICE: c_int = -8;
// TODO should be guarded by _GNU_SOURCE
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/getaddrinfo.3.html>.
/// The specified network host does not have any network adresses in the
/// requested address family.
pub const EAI_ADDRFAMILY: c_int = -9;
/// There was a memory allocation failure.
pub const EAI_MEMORY: c_int = -10;
/// A system error occurred. The error code can be found in errno.
pub const EAI_SYSTEM: c_int = -11;
/// An argument buffer overflowed.
pub const EAI_OVERFLOW: c_int = -12;

// TODO should be guarded by _GNU_SOURCE
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/getnameinfo.3.html>.
pub const NI_MAXHOST: c_int = 1025;
// TODO should be guarded by _GNU_SOURCE
/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/getnameinfo.3.html>.
pub const NI_MAXSERV: c_int = 32;

/// The numeric form of the node's address is returned instead of its name.
pub const NI_NUMERICHOST: c_int = 0x0001;
/// The numeric form of the service address is returned instead of its name.
pub const NI_NUMERICSERV: c_int = 0x0002;
/// Only the nodename portion of the FQDN is returned for local hosts.
pub const NI_NOFQDN: c_int = 0x0004;
/// Return an error if the node's name cannot be located in the database.
pub const NI_NAMEREQD: c_int = 0x0008;
/// Indicates that the service is a datagram service (`SOCK_DGRAM`).
pub const NI_DGRAM: c_int = 0x0010;

#[thread_local]
pub static H_ERRNO: Cell<c_int> = Cell::new(0);
const H_UNSET: c_int = 0;
/// Removed in POSIX issue 7.
pub const HOST_NOT_FOUND: c_int = 1;
/// Removed in POSIX issue 7.
pub const NO_DATA: c_int = 2;
/// Removed in POSIX issue 7.
pub const NO_RECOVERY: c_int = 3;
/// Removed in POSIX issue 7.
pub const TRY_AGAIN: c_int = 4;

// Expected length of addresses
const SOCKLEN_AF_INET4: socklen_t = 4;
const SOCKLEN_AF_INET6: socklen_t = 16;

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/endhostent.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn endhostent() {
    HOST_STATE.lock().endhostent();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/endnetent.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn endnetent() {
    NET_STATE.lock().endnetent();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/endprotoent.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn endprotoent() {
    PROTO_STATE.lock().endprotoent();
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/endservent.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn endservent() {
    SERV_STATE.lock().endservent();
}

/// See <https://pubs.opengroup.org/onlinepubs/009696799/functions/gethostbyaddr.html>.
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
#[unsafe(no_mangle)]
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
    if length != SOCKLEN_AF_INET4 || format != AF_INET {
        H_ERRNO.set(NO_RECOVERY);
        return ptr::null_mut();
    }
    let addr: in_addr = unsafe { (*(v as *mut in_addr)).clone() };

    // check the hosts file first
    if let Some(h) = HOST_STATE.lock().gethostbyaddr_local(addr.s_addr) {
        return h;
    }

    let r = lookup_addr(&addr);
    match HOST_STATE.lock().gethostbyaddr_external(r, addr) {
        Ok(h) => h,
        Err(e) => {
            H_ERRNO.set(e);
            core::ptr::null_mut()
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/009696799/functions/gethostbyaddr.html>.
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
#[unsafe(no_mangle)]
#[deprecated]
pub unsafe extern "C" fn gethostbyname(name: *const c_char) -> *mut hostent {
    #[expect(deprecated)]
    unsafe {
        gethostbyname2(name, AF_INET)
    }
}

/// Non-POSIX, see <https://man7.org/linux/man-pages/man3/gethostbyname.3.html>.
/// Resolve host information by name or IP address using AF_INET or AF_INET6.
///
/// # Deprecation
/// New code should use [`getaddrinfo`] instead.
#[unsafe(no_mangle)]
#[deprecated]
pub unsafe extern "C" fn gethostbyname2(name: *const c_char, af: c_int) -> *mut hostent {
    if af != AF_INET {
        // TODO: AF_INET6 support
        H_ERRNO.set(NO_RECOVERY);
        return ptr::null_mut();
    };
    let Some(name_cstr) = (unsafe { CStr::from_nullable_ptr(name) }) else {
        H_ERRNO.set(NO_RECOVERY);
        return ptr::null_mut();
    };
    let Ok(name_str) = str::from_utf8(name_cstr.to_bytes()) else {
        H_ERRNO.set(NO_RECOVERY);
        return ptr::null_mut();
    };

    // Addresses and hostnames are both valid, so we'll check addresses first
    // The standard doesn't define what to do when called with addresses
    // Some implementations just skip resolution and copy the address to h_name
    if let Some(s_addr) = parse_ipv4_string(name_str) {
        let addr = in_addr { s_addr };
        return unsafe {
            #[expect(deprecated)]
            gethostbyaddr(ptr::from_ref(&addr).cast::<c_void>(), 4, AF_INET)
        };
    }

    // check the hosts file first
    if let Some(h) = HOST_STATE
        .lock()
        .gethostbyname_local(name_cstr.to_bytes_with_nul())
    {
        return h;
    }

    let r = lookup_host(name_str);
    match HOST_STATE
        .lock()
        .gethostbyname_external(r, name_cstr.to_bytes_with_nul())
    {
        Ok(h) => h,
        Err(e) => {
            H_ERRNO.set(e);
            core::ptr::null_mut()
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/endhostent.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gethostent() -> *mut hostent {
    HOST_STATE
        .lock()
        .gethostent()
        .unwrap_or(core::ptr::null_mut())
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/endnetent.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getnetbyaddr(net: u32, net_type: c_int) -> *mut netent {
    match NET_STATE.lock().getnetbyaddr(net, net_type) {
        Some(n) => n,
        None => {
            crate::platform::ERRNO.set(ENOENT);
            core::ptr::null_mut()
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/endnetent.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getnetbyname(name: *const c_char) -> *mut netent {
    let name_bytes = unsafe { CStr::from_ptr(name) }.to_bytes_with_nul();
    match NET_STATE.lock().getnetbyname(name_bytes) {
        Some(n) => n,
        None => {
            crate::platform::ERRNO.set(ENOENT);
            core::ptr::null_mut()
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/endnetent.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getnetent() -> *mut netent {
    NET_STATE.lock().getnetent().unwrap_or(ptr::null_mut())
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/endprotoent.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getprotobyname(name: *const c_char) -> *mut protoent {
    let name_bytes = unsafe { CStr::from_ptr(name) }.to_bytes_with_nul();
    match PROTO_STATE.lock().getprotobyname(name_bytes) {
        Some(p) => p,
        None => {
            crate::platform::ERRNO.set(ENOENT);
            core::ptr::null_mut()
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/endprotoent.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getprotobynumber(number: c_int) -> *mut protoent {
    match PROTO_STATE.lock().getprotobynumber(number) {
        Some(p) => p,
        None => {
            crate::platform::ERRNO.set(ENOENT);
            core::ptr::null_mut()
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/endprotoent.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getprotoent() -> *mut protoent {
    PROTO_STATE
        .lock()
        .getprotoent()
        .unwrap_or(core::ptr::null_mut())
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/endservent.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getservbyname(name: *const c_char, proto: *const c_char) -> *mut servent {
    let name_bytes = unsafe { CStr::from_ptr(name) }.to_bytes_with_nul();
    let proto_bytes =
        (!proto.is_null()).then(|| unsafe { CStr::from_ptr(proto) }.to_bytes_with_nul());
    match SERV_STATE.lock().getservbyname(name_bytes, proto_bytes) {
        Some(s) => s,
        None => {
            crate::platform::ERRNO.set(ENOENT);
            core::ptr::null_mut()
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/endservent.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getservbyport(port: c_int, proto: *const c_char) -> *mut servent {
    let proto_bytes =
        (!proto.is_null()).then(|| unsafe { CStr::from_ptr(proto) }.to_bytes_with_nul());
    match SERV_STATE.lock().getservbyport(port, proto_bytes) {
        Some(s) => s,
        None => {
            crate::platform::ERRNO.set(ENOENT);
            core::ptr::null_mut()
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/endservent.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getservent() -> *mut servent {
    SERV_STATE
        .lock()
        .getservent()
        .unwrap_or(core::ptr::null_mut())
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/endhostent.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sethostent(stayopen: c_int) {
    HOST_STATE.lock().sethostent(stayopen != 0);
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/endnetent.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setnetent(stayopen: c_int) {
    NET_STATE.lock().setnetent(stayopen != 0);
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/endprotoent.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setprotoent(stayopen: c_int) {
    PROTO_STATE.lock().setprotoent(stayopen != 0);
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/endservent.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn setservent(stayopen: c_int) {
    SERV_STATE.lock().setservent(stayopen != 0);
}

fn getaddrinfo_inner(
    node_opt: Option<CStr>,
    service_opt: Option<CStr>,
    hints_opt: Option<&addrinfo>,
) -> Result<Box<addrinfo>, AddrInfoErr> {
    //TODO: Use hints
    let mut ai_flags = hints_opt.map_or(0, |hints| hints.ai_flags);
    let mut ai_family = hints_opt.map_or(AF_UNSPEC, |hints| hints.ai_family);
    let ai_socktype = hints_opt.map_or(0, |hints| hints.ai_socktype);
    let mut ai_protocol; // = hints_opt.map_or(0, |hints| hints.ai_protocol);

    if ai_family == AF_INET6 && ai_flags == AI_ADDRCONFIG {
        //TODO: support ipv6 lookup
        return Err(AddrInfoErr::EAI_NONAME);
    }

    let mut port = 0;
    if let Some(service) = service_opt {
        //TODO: Support other service definitions as well as AI_NUMERICSERV
        if let Ok(service) = str::from_utf8(service.to_bytes())
            && let Ok(ok) = service.parse::<u16>()
        {
            port = ok;
        }
    }
    let node_cstr = node_opt.unwrap_or_else(|| {
        //TODO: Optimize by bypassing string parsing
        if ai_flags & AI_PASSIVE > 0 {
            c"0.0.0.0".into()
        } else {
            c"127.0.0.1".into()
        }
    });
    let node = str::from_utf8(node_cstr.to_bytes()).map_err(|_| AddrInfoErr::EAI_NONAME)?;

    let lookuphost = if ai_flags & AI_NUMERICHOST > 0 {
        let s_addr = parse_ipv4_string(node).ok_or(AddrInfoErr::EAI_NONAME)?;

        vec![in_addr { s_addr }]
    } else {
        match lookup_host(node) {
            Ok(hosts) => hosts,
            Err(Errno(ETIMEDOUT)) => {
                return Err(AddrInfoErr::EAI_AGAIN);
            }
            Err(_) => return Err(AddrInfoErr::EAI_FAIL),
        }
    };

    let mut first_info: Option<Box<addrinfo>> = None;
    let mut last_info: Option<&mut addrinfo> = None;

    for in_addr in lookuphost {
        ai_family = AF_INET;
        ai_protocol = 0;

        let mut sockaddr = vec![0_u8; size_of::<sockaddr_in>()].into_boxed_slice();

        *plain::from_mut_bytes(&mut sockaddr).unwrap() = sockaddr_in {
            sin_family: sa_family_t::try_from(ai_family).expect("AF_INET is positive"),
            sin_port: htons(port),
            sin_addr: in_addr,
            sin_zero: [0; 8],
        };

        let ai_canonname = if ai_flags & AI_CANONNAME > 0 {
            if node_opt.is_none() {
                return Err(AddrInfoErr::EAI_BADFLAGS);
            }
            ai_flags &= !AI_CANONNAME;
            Some(OwnedThinCStr::from(node_cstr))
        } else {
            None
        };

        let addrinfo = Box::new(addrinfo::from_wrapped(WrappedAddrInfo {
            flags: ai_flags,
            family: ai_family,
            socktype: ai_socktype,
            protocol: ai_protocol,
            sockaddr,
            canonname: ai_canonname,
        }));

        if let Some(last) = last_info {
            let new_last = Box::leak(addrinfo);
            last.ai_next = new_last;
            last_info = Some(new_last);
        } else {
            first_info = Some(addrinfo);
            last_info = first_info.as_deref_mut();
        }
    }

    first_info.ok_or(AddrInfoErr::EAI_FAIL)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/freeaddrinfo.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getaddrinfo(
    node: *const c_char,
    service: *const c_char,
    hints: *const addrinfo,
    res_out: *mut *mut addrinfo,
) -> c_int {
    let node_opt = unsafe { CStr::from_nullable_ptr(node) };
    let service_opt = unsafe { CStr::from_nullable_ptr(service) };

    let hints_opt = if hints.is_null() {
        None
    } else {
        Some(unsafe { &*hints })
    };

    trace_log!(
        "getaddrinfo({:?}, {:?}, {:?})",
        node_opt,
        service_opt,
        hints_opt
    );

    match getaddrinfo_inner(node_opt, service_opt, hints_opt) {
        Ok(list_first) => unsafe {
            res_out.write(Box::into_raw(list_first));
            0
        },
        Err(err) => {
            unsafe {
                res_out.write(core::ptr::null_mut());
            }
            err as c_int
        }
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/getnameinfo.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn getnameinfo(
    addr: *const sockaddr,
    addrlen: socklen_t,
    host: *mut c_char,
    hostlen: socklen_t,
    serv: *mut c_char,
    servlen: socklen_t,
    flags: c_int,
) -> c_int {
    if addr.is_null() || addrlen as usize != mem::size_of::<sockaddr_in>() {
        return EAI_FAMILY;
    }

    let sa = unsafe { &*(addr.cast::<sockaddr_in>()) };

    if !serv.is_null() && servlen > 0 {
        if flags & NI_NUMERICSERV != 0 {
            let port_str = sa.sin_port.to_be().to_string();
            let port_bytes = port_str.as_bytes();
            if (servlen as usize) <= port_bytes.len() {
                return EAI_MEMORY; // Buffer too small
            }
            unsafe {
                ptr::copy_nonoverlapping(
                    port_bytes.as_ptr().cast::<c_char>(),
                    serv,
                    port_bytes.len(),
                )
            };
            unsafe { *serv.add(port_bytes.len()) = 0 };
        } else {
            // TODO: Implement service name lookup (e.g., from /etc/services)
            unsafe { *serv = 0 };
        }
    }

    if !host.is_null() && hostlen > 0 {
        if flags & NI_NUMERICHOST != 0 {
            let ip_addr = Ipv4Addr::from(sa.sin_addr.s_addr.to_be());
            let ip_str = ip_addr.to_string();
            let ip_bytes = ip_str.as_bytes();
            if (hostlen as usize) <= ip_bytes.len() {
                return EAI_MEMORY; // Buffer too small
            }
            unsafe {
                ptr::copy_nonoverlapping(ip_bytes.as_ptr().cast::<c_char>(), host, ip_bytes.len())
            };
            unsafe { *host.add(ip_bytes.len()) = 0 };
        } else {
            match lookup_addr(&sa.sin_addr).map(|host_names| host_names.into_iter().next()) {
                Ok(Some(hostname)) => {
                    if (hostlen as usize) <= hostname.len() {
                        return EAI_MEMORY; // Buffer too small
                    }
                    unsafe {
                        ptr::copy_nonoverlapping(
                            hostname.as_ptr().cast::<c_char>(),
                            host,
                            hostname.len(),
                        );
                        *host.add(hostname.len()) = 0;
                    }
                }
                Ok(None) => {
                    if flags & NI_NAMEREQD != 0 {
                        return EAI_NONAME;
                    }
                }
                Err(_) => {
                    if flags & NI_NAMEREQD != 0 {
                        return EAI_NONAME;
                    }
                    let ip_addr = Ipv4Addr::from(sa.sin_addr.s_addr.to_be());
                    let ip_str = ip_addr.to_string();
                    let ip_bytes = ip_str.as_bytes();
                    if (hostlen as usize) <= ip_bytes.len() {
                        return EAI_MEMORY;
                    }
                    unsafe {
                        ptr::copy_nonoverlapping(
                            ip_bytes.as_ptr().cast::<c_char>(),
                            host,
                            ip_bytes.len(),
                        );
                        *host.add(ip_bytes.len()) = 0;
                    }
                }
            }
        }
    }

    0
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/freeaddrinfo.html>.
#[unsafe(no_mangle)]
pub extern "C" fn freeaddrinfo(mut res: Box<addrinfo>) {
    // all deallocations are done automagically here using RAII
    while let (_wrapped, Some(next)) = res.into_wrapped() {
        res = next;
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/gai_strerror.html>.
#[unsafe(no_mangle)]
pub const extern "C" fn gai_strerror(errcode: c_int) -> *const c_char {
    match errcode {
        EAI_BADFLAGS => c"Invalid flags",
        EAI_NONAME => c"Name does not resolve",
        EAI_AGAIN => c"Try again",
        EAI_FAIL => c"Non-recoverable error",
        EAI_NODATA => c"Unknown error",
        EAI_FAMILY => c"Unrecognized address family or invalid length",
        EAI_SOCKTYPE => c"Unrecognized socket type",
        EAI_SERVICE => c"Unrecognized service",
        EAI_ADDRFAMILY => c"Address family for name not supported",
        EAI_MEMORY => c"Out of memory",
        EAI_SYSTEM => c"System error",
        EAI_OVERFLOW => c"Overflow",
        _ => c"Unknown error",
    }
    .as_ptr()
}

/// Provide a pointer to relibc's internal [`H_ERRNO`].
#[unsafe(no_mangle)]
#[deprecated]
pub extern "C" fn __h_errno_location() -> *mut c_int {
    H_ERRNO.as_ptr()
}

#[unsafe(no_mangle)]
#[deprecated]
pub const extern "C" fn hstrerror(errcode: c_int) -> *const c_char {
    match errcode {
        H_UNSET => c"Resolver error unset",
        HOST_NOT_FOUND => c"Unknown hostname",
        NO_DATA => c"No address for hostname",
        NO_RECOVERY => c"Unknown server error",
        TRY_AGAIN => c"Hostname lookup failure",
        _ => c"Unknown error",
    }
    .as_ptr()
}

/// Print error message associated with [`H_ERRNO`] to stderr.
///
/// # Arguments
/// * `prefix` - An optional prefix to prepend to the error message. May be null or an empty
///   (`""`) C string.
///
/// # Safety
/// Like [`crate::header::stdio::perror`], `prefix` should be a valid, NUL terminated C string if
/// used. The caller may safely call this function with a null pointer.
///
/// # Deprecation
/// [`H_ERRNO`], [`hstrerror`], [`herror`], and other functions are deprecated as of
/// POSIX.1-2001 and removed as of POSIX.1-2008. These functions are provided for backwards
/// compatibility but should not be used by new code.
#[expect(clippy::not_unsafe_ptr_arg_deref, reason = "see safety note")]
#[unsafe(no_mangle)]
#[deprecated]
pub extern "C" fn herror(prefix: *const c_char) {
    let code = H_ERRNO.get();
    // Safety: `hstrerror` handles every error code case and always returns a valid C string
    let error = unsafe {
        #[expect(deprecated)]
        let msg_cstr = CStr::from_ptr(hstrerror(code));
        str::from_utf8_unchecked(msg_cstr.to_bytes())
    };

    let mut writer = crate::platform::FileWriter::new(2);
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
