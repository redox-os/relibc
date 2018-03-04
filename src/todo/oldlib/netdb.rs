use libc;
use types::{in_addr, sockaddr, socklen_t};
use RawLineBuffer;
use core::ptr::null;
use core::{mem, str};
use alloc::vec::IntoIter;
use alloc::string::ToString;
use alloc::{Vec, String};
use alloc::str::SplitWhitespace;
use alloc::boxed::Box;
use dns::{Dns, DnsQuery};
use syscall::{self, Result, EINVAL, Error};
use libc::c_char;

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
    h_name: *const libc::c_char,
    h_aliases: *const *const libc::c_char,
    h_addrtype: libc::c_int,
    h_length: libc::c_int,
    h_addr_list: *const *const libc::c_char,
}

/*
 *#[repr(C)]
 *pub struct netent {
 *    n_name: *const libc::c_char, [> official name of net <]
 *    n_aliases: *const *const libc::c_char, [> alias list <]
 *    n_addrtype: libc::c_int, [> net address type <]
 *    n_net: libc::c_ulong, [> network # <]
 *}
 */

#[repr(C)]
pub struct servent {
    s_name: *const libc::c_char, /* official service name */
    s_aliases: *const *const libc::c_char, /* alias list */
    s_port: libc::c_int, /* port # */
    s_proto: *const libc::c_char, /* protocol to use */
}

#[repr(C)]
pub struct protoent {
    p_name: *const libc::c_char, /* official protocol name */
    p_aliases: *const *const libc::c_char, /* alias list */
    p_proto: libc::c_int, /* protocol # */
}

#[repr(C)]
pub struct addrinfo {
    ai_flags: libc::c_int, /* AI_PASSIVE, AI_CANONNAME, AI_NUMERICHOST */
    ai_family: libc::c_int, /* PF_xxx */
    ai_socktype: libc::c_int, /* SOCK_xxx */
    ai_protocol: libc::c_int, /* 0 or IPPROTO_xxx for IPv4 and IPv6 */
    ai_addrlen: libc::size_t, /* length of ai_addr */
    ai_canonname: *const libc::c_char, /* canonical name for hostname */
    ai_addr: *const sockaddr, /* binary address */
    ai_next: *const addrinfo, /* next structure in linked list */
}

static mut HOSTDB: usize = 0;
//static mut NETDB: usize = 0;
static mut PROTODB: usize = 0;
static mut SERVDB: usize = 0;

/*
 *static mut NET_ENTRY: netent = netent {
 *    n_name: 0 as *const libc::c_char,
 *    n_aliases: 0 as *const *const libc::c_char,
 *    n_addrtype: 0,
 *    n_net: 0 as u64,
 *};
 *static mut NET_NAME: Option<Vec<u8>> = None;
 *static mut NET_ALIASES: [*const c_char; MAXALIASES] = [null(); MAXALIASES];
 *static mut NET_NUM: Option<u64> = None;
 */

static mut HOST_ENTRY: hostent = hostent {
    h_name: 0 as *const libc::c_char,
    h_aliases: 0 as *const *const libc::c_char,
    h_addrtype: 0,
    h_length: 0,
    h_addr_list: 0 as *const *const libc::c_char,
};
static mut HOST_NAME: Option<Vec<u8>> = None;
static mut HOST_ALIASES: Option<Vec<Vec<u8>>> = None; 
static mut HOST_ADDR: Option<in_addr> = None;
static mut HOST_ADDR_LIST: [*const c_char; 2] = [null(); 2];
static mut H_LINE: RawLineBuffer = RawLineBuffer {
    fd: 0,
    cur: 0,
    read: 0,
    buf: [0; 8 * 1024],
};

static mut PROTO_ENTRY: protoent = protoent {
    p_name: 0 as *const libc::c_char,
    p_aliases: 0 as *const *const libc::c_char,
    p_proto: 0 as libc::c_int,
};
static mut PROTO_NAME: Option<Vec<u8>> = None;
static mut PROTO_ALIASES: Option<Vec<Vec<u8>>> = None;
static mut PROTO_NUM: Option<libc::c_int> = None;
static mut P_LINE: RawLineBuffer = RawLineBuffer {
    fd: 0,
    cur: 0,
    read: 0,
    buf: [0; 8 * 1024],
};

static mut SERV_ENTRY: servent = servent {
    s_name: 0 as *const libc::c_char,
    s_aliases: 0 as *const *const libc::c_char,
    s_port: 0 as libc::c_int,
    s_proto: 0 as *const libc::c_char,
};
static mut SERV_NAME: Option<Vec<u8>> = None;
static mut SERV_ALIASES: Option<Vec<Vec<u8>>> = None;
static mut SERV_PORT: Option<libc::c_int> = None;
static mut SERV_PROTO: Option<Vec<u8>> = None;
static mut S_LINE: RawLineBuffer = RawLineBuffer {
    fd: 0,
    cur: 0,
    read: 0,
    buf: [0; 8 * 1024],
};


fn lookup_host(host: &str) -> Result<LookupHost> {
    // XXX better error handling
    let ip_string = String::from_utf8(::file_read_all("/etc/net/ip")?).or(Err(
        Error::new(syscall::EIO),
    ))?;
    let ip: Vec<u8> = ip_string
        .trim()
        .split(".")
        .map(|part| part.parse::<u8>().unwrap_or(0))
        .collect();

    let dns_string = String::from_utf8(::file_read_all("/etc/net/dns")?).or(Err(
        Error::new(syscall::EIO),
    ))?;
    let dns: Vec<u8> = dns_string
        .trim()
        .split(".")
        .map(|part| part.parse::<u8>().unwrap_or(0))
        .collect();

    if ip.len() == 4 && dns.len() == 4 {
        let mut timespec = syscall::TimeSpec::default();
        syscall::clock_gettime(syscall::CLOCK_REALTIME, &mut timespec).unwrap();
        let tid = (timespec.tv_nsec >> 16) as u16;

        let packet = Dns {
            transaction_id: tid,
            flags: 0x0100,
            queries: vec![
                DnsQuery {
                    name: host.to_string(),
                    q_type: 0x0001,
                    q_class: 0x0001,
                },
            ],
            answers: vec![],
        };

        let packet_data = packet.compile();

        let fd = ::RawFile::open(
            format!("udp:/{}.{}.{}.{}:0", ip[0], ip[1], ip[2], ip[3]).as_bytes(),
            syscall::O_RDWR,
        )?;

        let timeout = syscall::TimeSpec {
            tv_sec: 5,
            tv_nsec: 0,
        };
        let rt = fd.dup(b"read_timeout")?;
        syscall::write(*rt, &timeout)?;
        drop(rt);
        let wt = fd.dup(b"write_timeout")?;
        syscall::write(*wt, &timeout)?;
        drop(wt);

        let sendrecvfd = fd.dup(
            format!("{}.{}.{}.{}:53", dns[0], dns[1], dns[2], dns[3])
                .as_bytes(),
        )?;
        syscall::write(*sendrecvfd, &packet_data)?;
        let mut buf = [0; 65536];
        let count = syscall::read(*sendrecvfd, &mut buf)?;
        drop(sendrecvfd);
        drop(fd);

        match Dns::parse(&buf[..count]) {
            Ok(response) => {
                let mut addrs = vec![];
                for answer in response.answers.iter() {
                    if answer.a_type == 0x0001 && answer.a_class == 0x0001 &&
                        answer.data.len() == 4
                    {
                        let addr = in_addr {
                            s_addr: [
                                answer.data[0],
                                answer.data[1],
                                answer.data[2],
                                answer.data[3],
                            ],
                        };
                        addrs.push(addr);
                    }
                }
                Ok(LookupHost(addrs.into_iter()))
            }
            Err(_err) => Err(Error::new(EINVAL)),
        }
    } else {
        Err(Error::new(EINVAL))
    }
}

unsafe fn lookup_addr(addr: in_addr) -> Result<Vec<Vec<u8>>> {
    // XXX better error handling
    let ip_string = String::from_utf8(::file_read_all("/etc/net/ip")?).or(Err(
        Error::new(syscall::EIO),
    ))?;
    let ip: Vec<u8> = ip_string
        .trim()
        .split(".")
        .map(|part| part.parse::<u8>().unwrap_or(0))
        .collect();

    let dns_string = String::from_utf8(::file_read_all("/etc/net/dns")?).or(Err(
        Error::new(syscall::EIO),
    ))?;
    let dns: Vec<u8> = dns_string
        .trim()
        .split(".")
        .map(|part| part.parse::<u8>().unwrap_or(0))
        .collect();
    
    let mut addr_vec: Vec<u8> = addr.s_addr.to_vec(); 
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
    let _ = syscall::write(2, name.as_slice());
    let _ = syscall::write(2, "\n".as_bytes());

    if ip.len() == 4 && dns.len() == 4 {
        let mut timespec = syscall::TimeSpec::default();
        syscall::clock_gettime(syscall::CLOCK_REALTIME, &mut timespec).unwrap();
        let tid = (timespec.tv_nsec >> 16) as u16;

        let packet = Dns {
            transaction_id: tid,
            flags: 0x0100,
            queries: vec![
                DnsQuery {
                    name: String::from_utf8(name).unwrap(),
                    q_type: 0x000C,
                    q_class: 0x0001,
                },
            ],
            answers: vec![],
        };

        let packet_data = packet.compile();

        let fd = ::RawFile::open(
            format!("udp:/{}.{}.{}.{}:0", ip[0], ip[1], ip[2], ip[3]).as_bytes(),
            syscall::O_RDWR,
        )?;

        let timeout = syscall::TimeSpec {
            tv_sec: 5,
            tv_nsec: 0,
        };
        let rt = fd.dup(b"read_timeout")?;
        syscall::write(*rt, &timeout)?;
        drop(rt);
        let wt = fd.dup(b"write_timeout")?;
        syscall::write(*wt, &timeout)?;
        drop(wt);

        let sendrecvfd = fd.dup(
            format!("{}.{}.{}.{}:53", dns[0], dns[1], dns[2], dns[3])
                .as_bytes(),
        )?;
        syscall::write(*sendrecvfd, &packet_data)?;
        let mut buf = [0; 65536];
        let count = syscall::read(*sendrecvfd, &mut buf)?;
        drop(sendrecvfd);
        drop(fd);

        match Dns::parse(&buf[..count]) {
            Ok(response) => {
                let _ = syscall::write(2, format!("{:?}", response).as_bytes());
                let _ = syscall::write(2, "\n".as_bytes());
                let mut names = vec![];
                for answer in response.answers.iter() {
     
                    if answer.a_type == 0x000C && answer.a_class == 0x0001 
                    {
                        // answer.data is encoded kinda weird.
                        // Basically length-prefixed strings for each 
                        // subsection of the domain. 
                        // We need to parse this to insert periods where
                        // they belong (ie at the end of each string)
                        let data = parse_data(answer.data.clone());
                        names.push(data);
                    }
                }
                Ok(names)
            }
            Err(_err) => Err(Error::new(EINVAL)),
        }
    } else {
        Err(Error::new(EINVAL))
    }
}

fn parse_data(mut data: Vec<u8>) -> Vec<u8> {
    let mut cursor = 0;
    let mut offset = 0;
    let mut index = 0; 
    let mut output = data.clone();
    while index < data.len() - 1 {
        offset = data[index] as usize;
        index = cursor + offset + 1;
        output[index] = '.' as u8;
        cursor = index;
    }
    //we don't want an extra period at the end
    output.pop(); 
    return output 
}



libc_fn!(unsafe endhostent() {
    let _ = syscall::close(HOSTDB);
});

/*
 *libc_fn!(unsafe endnetent()  {
 *    let _ = syscall::close(NETDB);
 *});
 */

libc_fn!(unsafe endprotoent() {
    let _ = syscall::close(PROTODB);
});

libc_fn!(unsafe endservent() {
    let _ = syscall::close(SERVDB);
});


libc_fn!(unsafe gethostbyaddr(v: *const libc::c_void, length: socklen_t, format: libc::c_int) -> Result <*const hostent> {
    let mut addr: in_addr = *(v as *mut in_addr);
    match lookup_addr(addr) {
        Ok(s) => {
            HOST_ADDR_LIST = [addr.s_addr.as_mut_ptr() as *const c_char, null()];
            let host_name = s[0].to_vec();
            HOST_ENTRY = hostent {
                h_name: host_name.as_ptr() as *const c_char,
                h_aliases: [null();2].as_mut_ptr(),
                h_addrtype: format,
                h_length: length as i32,
                h_addr_list: HOST_ADDR_LIST.as_ptr()
            };
            HOST_NAME = Some(host_name);
            return Ok(&HOST_ENTRY)
        }
        Err(err) => Err(err)
    }
});

libc_fn!(unsafe gethostbyname(name: *const c_char) -> Result<*const hostent> {
    // XXX h_errno
    let mut addr = mem::uninitialized();
    let mut host_addr = if ::socket::inet_aton(name, &mut addr) == 1 {
        addr
    } else {
        // XXX
        let mut host = lookup_host(str::from_utf8_unchecked(::cstr_to_slice(name)))?;
        host.next().ok_or(Error::new(syscall::ENOENT))? // XXX
    };

    let host_name: Vec<u8> = ::cstr_to_slice(name).to_vec();
    HOST_ADDR_LIST = [host_addr.s_addr.as_mut_ptr() as *const c_char, null()];
    HOST_ADDR = Some(host_addr);

    HOST_ENTRY = hostent {
        h_name: host_name.as_ptr() as *const c_char,
        h_aliases: [null();2].as_mut_ptr(),
        h_addrtype: ::socket::AF_INET,
        h_length: 4,
        h_addr_list: HOST_ADDR_LIST.as_ptr()
    };

    HOST_NAME = Some(host_name);

    Ok(&HOST_ENTRY as *const hostent)
});

libc_fn!(unsafe gethostent() -> *const hostent {
    if HOSTDB == 0 {
        HOSTDB = syscall::open("/etc/hosts", syscall::O_RDONLY).unwrap();
        H_LINE = RawLineBuffer::new(HOSTDB);
    } 

    let mut r: Box<str> = Box::default(); 
    while r.is_empty() || r.split_whitespace().next() == None || r.starts_with("#") {
        r = match H_LINE.next() {
            Some(Ok(s)) => s,
            Some(Err(_)) => return null(),
            None => return null(),
        };
    }


    let mut iter: SplitWhitespace = r.split_whitespace();
    
    let mut addr_vec = iter.next().unwrap().as_bytes().to_vec();
    addr_vec.push(b'\0');
    let addr_cstr = addr_vec.as_slice().as_ptr() as *const i8;
    let mut addr = mem::uninitialized();
    ::socket::inet_aton(addr_cstr, &mut addr);
    HOST_ADDR_LIST = [addr.s_addr.as_mut_ptr() as *const c_char, null()];
    HOST_ADDR = Some(addr);

    let mut host_name = iter.next().unwrap().as_bytes().to_vec();
    host_name.push(b'\0');
    
    let mut host_aliases: Vec<Vec<u8>> = Vec::new(); 
    
    loop {
        let mut alias = match iter.next() {
            Some(s) => s.as_bytes().to_vec(),
            None => break
        };
        alias.push(b'\0');
        host_aliases.push(alias);
    }

    //push a 0 so c doesn't segfault when it tries to read the next entry
    host_aliases.push(vec![b'\0']);

    HOST_ENTRY = hostent { 
        h_name: host_name.as_ptr() as *const c_char,
        h_aliases: host_aliases.as_slice().as_ptr() as *const *const i8,
        h_addrtype: ::socket::AF_INET, 
        h_length: 4,
        h_addr_list: HOST_ADDR_LIST.as_ptr()
    };
        HOST_ALIASES = Some(host_aliases);
        HOST_NAME = Some(host_name); 
        &HOST_ENTRY as *const hostent
});

/*
 *libc_fn!(getnetbyaddr(net: libc::uint32_t, net_type: libc::c_int) -> Result<*const netent> {
 *    if NETDB == 0 {
 *        NETDB = syscall::open("/etc/networks", syscall::O_RDONLY).unwrap();
 *    }
 *});
 *
 *libc_fn!(getnetbyname(name: *const libc::c_char) -> Result<*const netent> {
 *    if NETDB == 0 {
 *        NETDB = syscall::open("/etc/networks", syscall::O_RDONLY).unwrap();
 *    }
 *});
 *
 */


/*
 *libc_fn!(getnetent() -> Result<*const netent> {
 *    if NETDB == 0 {
 *        NETDB = syscall::open("/etc/networks", syscall::O_RDONLY).unwrap();
 *    }
 *
 *});
 */

libc_fn!(unsafe getprotobyname(name: *const libc::c_char) -> Result<*const protoent> {
    setprotoent(0);
    let mut p: *const protoent;
    while {p=getprotoent();
           p!=null()} {
        if libc::strcmp((*p).p_name, name) == 0 {
            return Ok(p);
        }
        loop {
            let mut cp = (*p).p_aliases;
            if cp == null() {
                break;
            }
			if libc::strcmp(*cp, name) == 0 {
			    return Ok(p);
			}
			cp = cp.offset(1);
	    }
    } 
    endprotoent();
    Err(Error::new(syscall::ENOENT))
});

libc_fn!(unsafe getprotobynumber(number: libc::c_int) -> Result<*const protoent> {
    setprotoent(0);
    let mut p: *const protoent;
    while {p=getprotoent();
           p!=null()} {
        if (*p).p_proto == number {
            return Ok(p);
        }
    }
    endprotoent();
    Err(Error::new(syscall::ENOENT))
 });

libc_fn!(unsafe getprotoent() -> *const protoent {
    if PROTODB == 0 {
        PROTODB = syscall::open("/etc/protocols", syscall::O_RDONLY).unwrap();
        P_LINE = RawLineBuffer::new(PROTODB);
    } 

    let mut r: Box<str> = Box::default(); 
    while r.is_empty() || r.split_whitespace().next() == None || r.starts_with("#") {
        r = match P_LINE.next() {
            Some(Ok(s)) => s,
            Some(Err(_)) => return null(),
            None => return null(),
        };
    }

    let mut iter: SplitWhitespace = r.split_whitespace();
    let mut proto_name: Vec<u8> = iter.next().unwrap().as_bytes().to_vec(); 
    proto_name.push(b'\0');
    
    let mut num = iter.next().unwrap().as_bytes().to_vec();
    num.push(b'\0');
    PROTO_NUM = Some(libc::atoi(num.as_slice().as_ptr() as *mut i8)); 

    let mut proto_aliases: Vec<Vec<u8>> = Vec::new(); 
    loop {
        let mut alias = match iter.next() {
            Some(s) => s.as_bytes().to_vec(),
            None => break
        };
        alias.push(b'\0');
        proto_aliases.push(alias);
    }
    //push a 0 so c doesn't segfault when it tries to read the next entry
    proto_aliases.push(vec![b'\0']);

    PROTO_ENTRY = protoent { 
        p_name: proto_name.as_slice().as_ptr() as *const c_char,
        p_aliases: proto_aliases.iter().map(|x| x.as_ptr() as *const i8).collect::<Vec<*const i8>>().as_ptr(),
        p_proto: PROTO_NUM.unwrap()           
    };
    PROTO_ALIASES = Some(proto_aliases);
    PROTO_NAME = Some(proto_name); 
    &PROTO_ENTRY as *const protoent
});

libc_fn!(unsafe getservbyname(name: *const libc::c_char, proto: *const libc::c_char) -> Result<*const servent> {
    setservent(0);
    let mut p: *const servent;
    while {p=getservent();
           p!=null()} {
        if libc::strcmp((*p).s_name, name) == 0 && libc::strcmp((*p).s_proto, proto) == 0 {
            return Ok(p);
        }
        loop {
            let mut cp = (*p).s_aliases;
            if cp == null() {
                break;
            }
			if libc::strcmp(*cp, name) == 0 && libc::strcmp((*p).s_proto, proto) == 0 {
			    return Ok(p);
			}
			cp = cp.offset(1);
	    }
    }
    Err(Error::new(syscall::ENOENT))
});

libc_fn!(unsafe getservbyport(port: libc::c_int, proto: *const libc::c_char) -> Result<*const servent> {
    setprotoent(0);
    let mut p: *const servent;
    while {p=getservent();
           p!=null()} {
        if (*p).s_port == port && libc::strcmp((*p).s_proto, proto) == 0 {
            return Ok(p);
        }
    }
    endprotoent();
    Err(Error::new(syscall::ENOENT))

});

libc_fn!(unsafe getservent() -> *const servent {
    if SERVDB == 0 {
        SERVDB = syscall::open("/etc/services", syscall::O_RDONLY).unwrap();
        S_LINE = RawLineBuffer::new(SERVDB);
    } 

    let mut r: Box<str> = Box::default(); 
    while r.is_empty() || r.split_whitespace().next() == None || r.starts_with("#") {
        r = match S_LINE.next() {
            Some(Ok(s)) => s,
            Some(Err(_)) => return null(),
            None => return null(),
        };
    }
    let mut iter: SplitWhitespace = r.split_whitespace();

    let mut serv_name: Vec<u8> = iter.next().unwrap().as_bytes().to_vec(); 
    serv_name.push(b'\0');

    let port_proto = iter.next().unwrap();
    let mut split = port_proto.split("/");
    let port = libc::atoi(split.next().unwrap().as_ptr() as *const i8);
    SERV_PORT = Some(port);
    let proto = split.next().unwrap().as_bytes().to_vec();

    let mut serv_aliases: Vec<Vec<u8>> = Vec::new(); 
    loop {
        let mut alias = match iter.next() {
            Some(s) => s.as_bytes().to_vec(),
            None => break
        };
        alias.push(b'\0');
        serv_aliases.push(alias);
    }
    //push a 0 so c doesn't segfault when it tries to read the next entry
    serv_aliases.push(vec![b'\0']);

    SERV_ENTRY = servent { 
        s_name: serv_name.as_slice().as_ptr() as *const c_char,
        s_aliases: serv_aliases.iter().map(|x| x.as_ptr() as *const i8).collect::<Vec<*const i8>>().as_ptr(),
        s_port: SERV_PORT.unwrap(),
        s_proto: proto.as_slice().as_ptr() as *const c_char            
    };

    SERV_ALIASES = Some(serv_aliases);
    SERV_NAME = Some(serv_name); 
    SERV_PROTO = Some(proto);
    &SERV_ENTRY as *const servent

});

libc_fn!(unsafe sethostent(stayopen: libc::c_int)  {
    if HOSTDB == 0 {
        HOSTDB = syscall::open("/etc/hosts", syscall::O_RDONLY).unwrap();
    } else {
        let _ = syscall::lseek(HOSTDB, 0, syscall::SEEK_SET);
    }
    H_LINE = RawLineBuffer::new(HOSTDB);
});

/*
 *libc_fn!(unsafe setnetent(stayopen: libc::c_int)  {
 *    if NETDB == 0 {
 *        NETDB = syscall::open("/etc/networks", syscall::O_RDONLY).unwrap();
 *    } else {
 *        let _ = syscall::lseek(NETDB, 0, syscall::SEEK_SET);
 *    }
 *});
 */

libc_fn!(unsafe setprotoent(stayopen: libc::c_int)  {
    if PROTODB == 0 {
        PROTODB = syscall::open("/etc/protocols", syscall::O_RDONLY).unwrap();
    } else {
        let _ = syscall::lseek(PROTODB, 0, syscall::SEEK_SET);
    }
    P_LINE = RawLineBuffer::new(PROTODB);
});

libc_fn!(unsafe setservent(stayopen: libc::c_int)  {
    if SERVDB == 0 {
        SERVDB = syscall::open("/etc/services", syscall::O_RDONLY).unwrap();
    } else {
        let _ = syscall::lseek(SERVDB, 0, syscall::SEEK_SET);
    }
    S_LINE = RawLineBuffer::new(SERVDB);
});

//libc_fn!(getaddrinfo(node: *const libc::c_char, service: *const libc::c_char, hints: *const addrinfo, res: *mut *mut addrinfo) -> libc::c_int {

//});

//libc_fn!(getnameinfo(addr: *const sockaddr, addrlen: socklen_t, host: *mut libc::c_char, hostlen: socklen_t, serv: *mut libc::c_char, servlen: socklen_t, flags: libc::c_int) -> libc::c_int {

//});

//libc_fn!(freeaddrinfo(res: *mut addrinfo)  {

//});

//libc_fn!(gai_strerror(errcode: libc::c_int) -> *const libc::c_char {

//});
