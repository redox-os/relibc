//! netdb implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xns/netdb.h.html

mod dns;

use core::str::FromStr;
use core::{mem, ptr, str};

use alloc::boxed::Box;
use alloc::str::SplitWhitespace;
use alloc::string::{String, ToString};
use alloc::vec::{IntoIter, Vec};

use c_str::{CStr, CString};

use platform;
use platform::rlb::{Line, RawLineBuffer};
use platform::types::*;
use platform::{Pal, Sys};

use self::dns::{Dns, DnsQuery};

use header::arpa_inet::{htons, inet_aton};
use header::errno::*;
use header::fcntl::O_RDONLY;
use header::netinet_in::{in_addr, sockaddr_in, IPPROTO_UDP};
use header::stdlib::atoi;
use header::strings::strcasecmp;
use header::sys_socket;
use header::sys_socket::constants::{AF_INET, SOCK_DGRAM};
use header::sys_socket::{sockaddr, socklen_t};
use header::time;
use header::time::timespec;
use header::unistd::SEEK_SET;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
pub mod sys;

#[cfg(target_os = "redox")]
#[path = "redox.rs"]
pub mod sys;

const MAXADDRS: usize = 35;
const MAXALIASES: usize = 35;

struct LookupHost(IntoIter<in_addr>);

impl Iterator for LookupHost {
    type Item = in_addr;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

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
pub struct servent {
    s_name: *mut c_char,         /* official service name */
    s_aliases: *mut *mut c_char, /* alias list */
    s_port: c_int,               /* port # */
    s_proto: *mut c_char,        /* protocol to use */
}

#[repr(C)]
pub struct protoent {
    p_name: *mut c_char,         /* official protocol name */
    p_aliases: *mut *mut c_char, /* alias list */
    p_proto: c_int,              /* protocol # */
}

#[repr(C)]
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

static mut NETDB: c_int = 0;
static mut NET_ENTRY: netent = netent {
    n_name: ptr::null_mut(),
    n_aliases: ptr::null_mut(),
    n_addrtype: 0,
    n_net: 0,
};
static mut NET_NAME: Option<Vec<u8>> = None;
static mut NET_ALIASES: [*const c_char; MAXALIASES] = [ptr::null(); MAXALIASES];
static mut NET_NUM: Option<u64> = None;
static mut N_POS: usize = 0;
static mut NET_STAYOPEN: c_int = 0;

static mut HOSTDB: c_int = 0;
static mut HOST_ENTRY: hostent = hostent {
    h_name: ptr::null_mut(),
    h_aliases: ptr::null_mut(),
    h_addrtype: 0,
    h_length: 0,
    h_addr_list: ptr::null_mut(),
};
static mut HOST_NAME: Option<Vec<u8>> = None;
static mut HOST_ALIASES: Option<Vec<Vec<u8>>> = None;
static mut _HOST_ALIASES: Option<Vec<*mut i8>> = None;
static mut HOST_ADDR: Option<in_addr> = None;
static mut HOST_ADDR_LIST: [*mut c_char; 2] = [ptr::null_mut(); 2];
static mut _HOST_ADDR_LIST: [u8; 4] = [0u8; 4];
static mut H_POS: usize = 0;
static mut HOST_STAYOPEN: c_int = 0;

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut h_errno: c_int = 0;
pub const HOST_NOT_FOUND: c_int = 1;
pub const NO_DATA: c_int = 2;
pub const NO_RECOVERY: c_int = 3;
pub const TRY_AGAIN: c_int = 4;

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

const NULL_ALIASES: [*mut c_char; 2] = [ptr::null_mut(); 2];

fn bytes_to_box_str(bytes: &[u8]) -> Box<str> {
    Box::from(core::str::from_utf8(bytes).unwrap_or(""))
}

fn lookup_host(host: &str) -> Result<LookupHost, c_int> {
    let dns_string = sys::get_dns_server();

    let dns_vec: Vec<u8> = dns_string
        .trim()
        .split(".")
        .map(|octet| octet.parse::<u8>().unwrap_or(0))
        .collect();

    if dns_vec.len() == 4 {
        let mut dns_arr = [0u8; 4];
        for (i, octet) in dns_vec.iter().enumerate() {
            dns_arr[i] = *octet;
        }
        let dns_addr = unsafe { mem::transmute::<[u8; 4], u32>(dns_arr) };

        let mut timespec = timespec::default();
        Sys::clock_gettime(time::constants::CLOCK_REALTIME, &mut timespec);
        let tid = (timespec.tv_nsec >> 16) as u16;

        let packet = Dns {
            transaction_id: tid,
            flags: 0x0100,
            queries: vec![DnsQuery {
                name: host.to_string(),
                q_type: 0x0001,
                q_class: 0x0001,
            }],
            answers: vec![],
        };

        let packet_data = packet.compile();
        let packet_data_len = packet_data.len();

        let packet_data_box = packet_data.into_boxed_slice();
        let packet_data_ptr = Box::into_raw(packet_data_box) as *mut _ as *mut c_void;

        let dest = sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: htons(53),
            sin_addr: in_addr { s_addr: dns_addr },
            ..Default::default()
        };
        let dest_ptr = &dest as *const _ as *const sockaddr;

        let sock = unsafe {
            let sock = sys_socket::socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP as i32);
            if sys_socket::connect(sock, dest_ptr, mem::size_of_val(&dest) as socklen_t) < 0 {
                return Err(EIO);
            }
            if sys_socket::send(sock, packet_data_ptr, packet_data_len, 0) < 0 {
                Box::from_raw(packet_data_ptr);
                return Err(EIO);
            }
            sock
        };

        unsafe {
            Box::from_raw(packet_data_ptr);
        }

        let i = 0 as socklen_t;
        let mut buf = [0u8; 65536];
        let buf_ptr = buf.as_mut_ptr() as *mut c_void;

        let count = unsafe { sys_socket::recv(sock, buf_ptr, 65536, 0) };
        if count < 0 {
            return Err(EIO);
        }

        match Dns::parse(&buf[..count as usize]) {
            Ok(response) => {
                let mut addrs = vec![];
                for answer in response.answers.iter() {
                    if answer.a_type == 0x0001 && answer.a_class == 0x0001 && answer.data.len() == 4
                    {
                        let addr = in_addr {
                            s_addr: unsafe {
                                mem::transmute::<[u8; 4], u32>([
                                    answer.data[0],
                                    answer.data[1],
                                    answer.data[2],
                                    answer.data[3],
                                ])
                            },
                        };
                        addrs.push(addr);
                    }
                }
                Ok(LookupHost(addrs.into_iter()))
            }
            Err(_err) => Err(EINVAL),
        }
    } else {
        Err(EINVAL)
    }
}

fn lookup_addr(addr: in_addr) -> Result<Vec<Vec<u8>>, c_int> {
    let dns_string = sys::get_dns_server();

    let dns_vec: Vec<u8> = dns_string
        .trim()
        .split('.')
        .map(|octet| octet.parse::<u8>().unwrap_or(0))
        .collect();

    let mut dns_arr = [0u8; 4];

    for (i, octet) in dns_vec.iter().enumerate() {
        dns_arr[i] = *octet;
    }

    let mut addr_vec: Vec<u8> = unsafe { mem::transmute::<u32, [u8; 4]>(addr.s_addr).to_vec() };
    addr_vec.reverse();
    let mut name: Vec<u8> = vec![];
    for octet in addr_vec {
        for ch in format!("{}", octet).as_bytes() {
            name.push(*ch);
        }
        name.push(b"."[0]);
    }
    name.pop();
    for ch in b".IN-ADDR.ARPA" {
        name.push(*ch);
    }

    if dns_vec.len() == 4 {
        let mut timespec = timespec::default();
        Sys::clock_gettime(time::constants::CLOCK_REALTIME, &mut timespec);
        let tid = (timespec.tv_nsec >> 16) as u16;

        let packet = Dns {
            transaction_id: tid,
            flags: 0x0100,
            queries: vec![DnsQuery {
                name: String::from_utf8(name).unwrap(),
                q_type: 0x000C,
                q_class: 0x0001,
            }],
            answers: vec![],
        };

        let packet_data = packet.compile();
        let packet_data_len = packet_data.len();
        let packet_data_box = packet_data.into_boxed_slice();
        let packet_data_ptr = Box::into_raw(packet_data_box) as *mut _ as *mut c_void;

        let dest = sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: htons(53),
            sin_addr: in_addr {
                s_addr: unsafe { mem::transmute::<[u8; 4], u32>(dns_arr) },
            },
            ..Default::default()
        };

        let dest_ptr = &dest as *const _ as *const sockaddr;

        let sock = unsafe {
            let sock = sys_socket::socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP as i32);
            if sys_socket::connect(sock, dest_ptr, mem::size_of_val(&dest) as socklen_t) < 0 {
                return Err(EIO);
            }
            sock
        };

        unsafe {
            if sys_socket::send(sock, packet_data_ptr, packet_data_len, 0) < 0 {
                return Err(EIO);
            }
        }

        unsafe {
            Box::from_raw(packet_data_ptr);
        }

        let i = mem::size_of::<sockaddr_in>() as socklen_t;
        let mut buf = [0u8; 65536];
        let buf_ptr = buf.as_mut_ptr() as *mut c_void;

        let count = unsafe { sys_socket::recv(sock, buf_ptr, 65536, 0) };
        if count < 0 {
            return Err(EIO);
        }

        match Dns::parse(&buf[..count as usize]) {
            Ok(response) => {
                let mut names = vec![];
                for answer in response.answers.iter() {
                    if answer.a_type == 0x000C && answer.a_class == 0x0001 {
                        // answer.data is encoded kinda weird.
                        // Basically length-prefixed strings for each
                        // subsection of the domain.
                        // We need to parse this to insert periods where
                        // they belong (ie at the end of each string)
                        let data = parse_revdns_answer(&answer.data);
                        names.push(data);
                    }
                }
                Ok(names)
            }
            Err(_err) => Err(EINVAL),
        }
    } else {
        Err(EINVAL)
    }
}

fn parse_revdns_answer(data: &[u8]) -> Vec<u8> {
    let mut cursor = 0;
    let mut index = 0;
    let mut output = data.to_vec();
    while index < data.len() - 1 {
        let offset = data[index] as usize;
        index = cursor + offset + 1;
        output[index] = b'.';
        cursor = index;
    }
    //we don't want an extra period at the end
    output.pop();
    output
}

#[no_mangle]
pub unsafe extern "C" fn endhostent() {
    Sys::close(HOSTDB);
    HOSTDB = 0;
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

#[no_mangle]
pub unsafe extern "C" fn gethostbyaddr(
    v: *const c_void,
    length: socklen_t,
    format: c_int,
) -> *mut hostent {
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

    match lookup_addr(addr) {
        Ok(s) => {
            _HOST_ADDR_LIST = mem::transmute::<u32, [u8; 4]>(addr.s_addr);
            HOST_ADDR_LIST = [_HOST_ADDR_LIST.as_mut_ptr() as *mut c_char, ptr::null_mut()];
            let mut host_name = s[0].to_vec();
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
        Err(e) => {
            platform::errno = e;
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn gethostbyname(name: *const c_char) -> *mut hostent {
    // check if some idiot gave us an address instead of a name
    let name_cstr = CStr::from_ptr(name);
    let mut octets = str::from_utf8_unchecked(name_cstr.to_bytes()).split('.');
    let mut s_addr = [0u8; 4];
    let mut is_addr = true;
    for item in &mut s_addr {
        if let Some(n) = octets.next().and_then(|x| u8::from_str(x).ok()) {
            *item = n;
        } else {
            is_addr = false;
        }
    }
    if octets.next() != None {
        is_addr = false;
    }

    if is_addr {
        let addr = in_addr {
            s_addr: mem::transmute::<[u8; 4], u32>(s_addr),
        };
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

    let name_cstr = CStr::from_ptr(name);

    let mut host = match lookup_host(str::from_utf8_unchecked(name_cstr.to_bytes())) {
        Ok(lookuphost) => lookuphost,
        Err(e) => {
            platform::errno = e;
            return ptr::null_mut();
        }
    };
    let host_addr = match host.next() {
        Some(result) => result,
        None => {
            platform::errno = ENOENT;
            return ptr::null_mut();
        }
    };

    let host_name: Vec<u8> = name_cstr.to_bytes().to_vec();
    HOST_NAME = Some(host_name);
    _HOST_ADDR_LIST = mem::transmute::<u32, [u8; 4]>(host_addr.s_addr);
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

#[no_mangle]
pub unsafe extern "C" fn gethostent() -> *mut hostent {
    if HOSTDB == 0 {
        HOSTDB = Sys::open(&CString::new("/etc/hosts").unwrap(), O_RDONLY, 0);
    }
    let mut rlb = RawLineBuffer::new(HOSTDB);
    rlb.seek(H_POS);

    let mut r: Box<str> = Box::default();
    while r.is_empty() || r.split_whitespace().next() == None || r.starts_with('#') {
        r = match rlb.next() {
            Line::Some(s) => bytes_to_box_str(s),
            _ => {
                if HOST_STAYOPEN == 0 {
                    endhostent();
                }
                return ptr::null_mut();
            }
        };
    }
    rlb.next();
    H_POS = rlb.line_pos();

    let mut iter: SplitWhitespace = r.split_whitespace();

    let mut addr_vec = iter.next().unwrap().as_bytes().to_vec();
    addr_vec.push(b'\0');
    let addr_cstr = addr_vec.as_slice().as_ptr() as *const i8;
    let mut addr = mem::uninitialized();
    inet_aton(addr_cstr, &mut addr);

    _HOST_ADDR_LIST = mem::transmute::<u32, [u8; 4]>(addr.s_addr);
    HOST_ADDR_LIST = [_HOST_ADDR_LIST.as_mut_ptr() as *mut c_char, ptr::null_mut()];

    HOST_ADDR = Some(addr);

    let mut host_name = iter.next().unwrap().as_bytes().to_vec();
    host_name.push(b'\0');

    let mut _host_aliases: Vec<Vec<u8>> = Vec::new();

    while let Some(s) = iter.next() {
        let mut alias = s.as_bytes().to_vec();
        alias.push(b'\0');
        _host_aliases.push(alias);
    }
    HOST_ALIASES = Some(_host_aliases);

    let mut host_aliases: Vec<*mut i8> = HOST_ALIASES
        .as_mut()
        .unwrap()
        .iter_mut()
        .map(|x| x.as_mut_ptr() as *mut i8)
        .collect();
    host_aliases.push(ptr::null_mut());
    host_aliases.push(ptr::null_mut());

    HOST_NAME = Some(host_name);

    HOST_ENTRY = hostent {
        h_name: HOST_NAME.as_mut().unwrap().as_mut_ptr() as *mut c_char,
        h_aliases: host_aliases.as_mut_slice().as_mut_ptr() as *mut *mut i8,
        h_addrtype: AF_INET,
        h_length: 4,
        h_addr_list: HOST_ADDR_LIST.as_mut_ptr(),
    };
    _HOST_ALIASES = Some(host_aliases);
    if HOST_STAYOPEN == 0 {
        endhostent();
    }
    &mut HOST_ENTRY as *mut hostent
}

pub unsafe extern "C" fn getnetbyaddr(net: u32, net_type: c_int) -> *mut netent {
    unimplemented!();
}

pub unsafe extern "C" fn getnetbyname(name: *const c_char) -> *mut netent {
    unimplemented!();
}

pub unsafe extern "C" fn getnetent() -> *mut netent {
    unimplemented!();
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

    platform::errno = ENOENT;
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
    platform::errno = ENOENT;
    ptr::null_mut() as *mut protoent
}

#[no_mangle]
pub unsafe extern "C" fn getprotoent() -> *mut protoent {
    if PROTODB == 0 {
        PROTODB = Sys::open(&CString::new("/etc/protocols").unwrap(), O_RDONLY, 0);
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

    let mut _proto_aliases: Vec<Vec<u8>> = Vec::new();
    while let Some(s) = iter.next() {
        let mut alias = s.as_bytes().to_vec();
        alias.push(b'\0');
        _proto_aliases.push(alias);
    }
    let mut proto_aliases: Vec<*mut i8> = _proto_aliases
        .iter_mut()
        .map(|x| x.as_mut_ptr() as *mut i8)
        .collect();
    proto_aliases.push(ptr::null_mut());

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
pub unsafe extern "C" fn getservbyname(
    name: *const c_char,
    proto: *const c_char,
) -> *mut servent {
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
    platform::errno = ENOENT;
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
    platform::errno = ENOENT;
    ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn getservent() -> *mut servent {
    if SERVDB == 0 {
        SERVDB = Sys::open(&CString::new("/etc/services").unwrap(), O_RDONLY, 0);
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
        let mut serv_name = match iter.next() {
            Some(serv_name) => serv_name.as_bytes().to_vec(),
            None => continue,
        };
        serv_name.push(b'\0');
        let port_proto = match iter.next() {
            Some(port_proto) => port_proto,
            None => continue,
        };
        let mut split = port_proto.split('/');
        let mut port = match split.next() {
            Some(port) => port.as_bytes().to_vec(),
            None => continue,
        };
        port.push(b'\0');
        SERV_PORT =
            Some(htons(atoi(port.as_mut_slice().as_mut_ptr() as *mut i8) as u16) as u32 as i32);
        let mut proto = match split.next() {
            Some(proto) => proto.as_bytes().to_vec(),
            None => continue,
        };
        proto.push(b'\0');

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
pub unsafe extern "C" fn sethostent(stayopen: c_int) {
    HOST_STAYOPEN = stayopen;
    if HOSTDB == 0 {
        HOSTDB = Sys::open(&CString::new("/etc/hosts").unwrap(), O_RDONLY, 0)
    } else {
        Sys::lseek(HOSTDB, 0, SEEK_SET);
    }
    H_POS = 0;
}

#[no_mangle]
pub unsafe extern "C" fn setnetent(stayopen: c_int) {
    NET_STAYOPEN = stayopen;
    if NETDB == 0 {
        NETDB = Sys::open(&CString::new("/etc/networks").unwrap(), O_RDONLY, 0)
    } else {
        Sys::lseek(NETDB, 0, SEEK_SET);
        N_POS = 0;
    }
}

#[no_mangle]
pub unsafe extern "C" fn setprotoent(stayopen: c_int) {
    PROTO_STAYOPEN = stayopen;
    if PROTODB == 0 {
        PROTODB = Sys::open(&CString::new("/etc/protocols").unwrap(), O_RDONLY, 0)
    } else {
        Sys::lseek(PROTODB, 0, SEEK_SET);
        P_POS = 0;
    }
}

#[no_mangle]
pub unsafe extern "C" fn setservent(stayopen: c_int) {
    SERV_STAYOPEN = stayopen;
    if SERVDB == 0 {
        SERVDB = Sys::open(&CString::new("/etc/services").unwrap(), O_RDONLY, 0)
    } else {
        Sys::lseek(SERVDB, 0, SEEK_SET);
        S_POS = 0;
    }
}

pub unsafe extern "C" fn getaddrinfo(
    node: *const c_char,
    service: *const c_char,
    hints: *const addrinfo,
    res: *mut *mut addrinfo,
) -> c_int {
    unimplemented!();
}

pub unsafe extern "C" fn getnameinfo(
    addr: *const sockaddr,
    addrlen: socklen_t,
    host: *mut c_char,
    hostlen: socklen_t,
    serv: *mut c_char,
    servlen: socklen_t,
    flags: c_int,
) -> c_int {
    unimplemented!();
}

pub extern "C" fn freeaddrinfo(res: *mut addrinfo) {
    unimplemented!();
}

pub extern "C" fn gai_strerror(errcode: c_int) -> *const c_char {
    unimplemented!();
}
