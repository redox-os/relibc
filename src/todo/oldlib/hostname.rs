use core::ptr::null;
use core::{mem, str, slice};
use alloc::vec::IntoIter;
use alloc::string::ToString;
use alloc::{Vec, String};
use ::dns::{Dns, DnsQuery};
use syscall::{self, Result, EINVAL, Error};
use libc::{c_char, size_t, c_int};
use ::types::{in_addr, hostent};

static mut HOST_ENTRY: hostent = hostent { h_name: null(), h_aliases: null(), h_addrtype: 0, h_length: 0, h_addr_list: null() };
static mut HOST_NAME: Option<Vec<u8>> = None;
static mut HOST_ALIASES: [*const c_char; 1] = [null()];
static mut HOST_ADDR: Option<in_addr> = None;
static mut HOST_ADDR_LIST: [*const c_char; 2] = [null(); 2];

struct LookupHost(IntoIter<in_addr>);

impl Iterator for LookupHost {
    type Item = in_addr;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

// Modified from rust/sys/redox/net/mod.rs
fn lookup_host(host: &str) -> Result<LookupHost> {
    // XXX better error handling
    let ip_string = String::from_utf8(::file_read_all("/etc/net/ip")?)
        .or(Err(Error::new(syscall::EIO)))?;
    let ip: Vec<u8> = ip_string.trim().split(".").map(|part| part.parse::<u8>()
                               .unwrap_or(0)).collect();

    let dns_string = String::from_utf8(::file_read_all("/etc/net/dns")?)
        .or(Err(Error::new(syscall::EIO)))?;
    let dns: Vec<u8> = dns_string.trim().split(".").map(|part| part.parse::<u8>()
                                 .unwrap_or(0)).collect();

    if ip.len() == 4 && dns.len() == 4 {
        let mut timespec = syscall::TimeSpec::default();
        syscall::clock_gettime(syscall::CLOCK_REALTIME, &mut timespec).unwrap();
        let tid = (timespec.tv_nsec >> 16) as u16;

        let packet = Dns {
            transaction_id: tid,
            flags: 0x0100,
            queries: vec![DnsQuery {
                name: host.to_string(),
                q_type: 0x0001,
                q_class: 0x0001,
            }],
            answers: vec![]
        };

        let packet_data = packet.compile();

        let fd = ::RawFile::open(format!("udp:/{}.{}.{}.{}:0",
                                         ip[0], ip[1], ip[2], ip[3]).as_bytes(),
                                 syscall::O_RDWR)?;

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

        let sendrecvfd = fd.dup(format!("{}.{}.{}.{}:53", dns[0], dns[1], dns[2], dns[3]).as_bytes())?;
        syscall::write(*sendrecvfd, &packet_data)?;
        let mut buf = [0; 65536];
        let count = syscall::read(*sendrecvfd, &mut buf)?;
        drop(sendrecvfd);
        drop(fd);

        match Dns::parse(&buf[.. count]) {
            Ok(response) => {
                let mut addrs = vec![];
                for answer in response.answers.iter() {
                    if answer.a_type == 0x0001 && answer.a_class == 0x0001
                       && answer.data.len() == 4
                    {
                        let addr = in_addr {
                            s_addr: [answer.data[0], answer.data[1], answer.data[2], answer.data[3]]
                        };
                        addrs.push(addr);
                    }
                }
                Ok(LookupHost(addrs.into_iter()))
            },
            Err(_err) => Err(Error::new(EINVAL))
        }
    } else {
        Err(Error::new(EINVAL))
    }
}

libc_fn!(unsafe gethostname(name: *mut c_char, namelen: size_t) -> Result<c_int> {
    let slice = slice::from_raw_parts_mut(name as *mut u8, namelen);
    let fd = ::RawFile::open("/etc/hostname", syscall::O_RDONLY)?;
    let len = syscall::read(*fd, &mut slice[..namelen-1])?;
    slice[len] = b'\0';
    Ok(0)
});
