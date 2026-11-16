use core::ptr;

use alloc::{string::ToString, vec::Vec};

use crate::{
    header::{
        arpa_inet::inet_aton_inner, fcntl::O_RDONLY, netdb::hostent, netinet_in::in_addr,
        stdio::SEEK_SET, sys_socket::constants::AF_INET,
    },
    platform::{
        Line, Pal, RawLineBuffer, Sys,
        types::{c_char, c_int},
    },
    raw_cell::RawCell,
    sync::Mutex,
};

static mut HOSTDB: c_int = -1;
pub static mut HOST_ENTRY: hostent = hostent {
    h_name: ptr::null_mut(),
    h_aliases: ptr::null_mut(),
    h_addrtype: 0,
    h_length: 0,
    h_addr_list: ptr::null_mut(),
};
pub static HOST_NAME: RawCell<Option<Vec<u8>>> = RawCell::new(None);
pub static HOST_ALIASES: RawCell<Option<Vec<Vec<u8>>>> = RawCell::new(None);
static _HOST_ALIASES: RawCell<Option<Vec<*mut c_char>>> = RawCell::new(None);
pub static mut HOST_ADDR: Option<in_addr> = None;
pub static mut HOST_ADDR_LIST: [*mut c_char; 2] = [ptr::null_mut(); 2];
pub static mut _HOST_ADDR_LIST: [u8; 4] = [0u8; 4];
static mut H_POS: usize = 0;
pub static mut HOST_STAYOPEN: c_int = 0;

pub struct HostState {
    db: Option<c_int>,
    rlb: Option<RawLineBuffer>,
    pub stayopen: bool,

    name: Vec<u8>,
    aliases: Vec<Vec<u8>>,
    aliases_ptrs: Vec<*mut c_char>,
    addr_bytes: [u8; 4],
    addr_list: [*mut c_char; 2],
    entry: hostent,
}

impl HostState {
    const fn new() -> Self {
        Self {
            db: None,
            rlb: None,
            stayopen: false,
            name: Vec::new(),
            aliases: Vec::new(),
            aliases_ptrs: Vec::new(),
            addr_bytes: [0; 4],
            addr_list: [core::ptr::null_mut(); 2],
            entry: hostent {
                h_name: core::ptr::null_mut(),
                h_aliases: core::ptr::null_mut(),
                h_addrtype: 0,
                h_length: 0,
                h_addr_list: core::ptr::null_mut(),
            },
        }
    }

    pub fn endhostent_def(&mut self) {
        if !self.stayopen {
            self.endhostent();
        }
    }

    pub fn sethostent_def(&mut self) {
        self.sethostent(self.stayopen);
    }

    pub fn endhostent(&mut self) {
        self.rlb = None;
        if let Some(fd) = self.db.take() {
            let _ = Sys::close(fd);
        }
    }

    pub fn sethostent(&mut self, stayopen: bool) {
        self.stayopen = stayopen;
        if self.db.is_none() {
            if let Ok(fd) = Sys::open(c"/etc/hosts".into(), O_RDONLY, 0) {
                self.db = Some(fd);
                self.rlb = Some(RawLineBuffer::new(fd));
            }
        } else if let Some(fd) = self.db {
            let _ = Sys::lseek(fd, 0, SEEK_SET);
            self.rlb = Some(RawLineBuffer::new(fd));
        }
    }

    pub fn gethostent(&mut self) -> Option<*mut hostent> {
        if self.rlb.is_none() {
            if let Ok(fd) = Sys::open(c"/etc/hosts".into(), O_RDONLY, 0) {
                self.db = Some(fd);
                self.rlb = Some(RawLineBuffer::new(fd));
            } else {
                return None;
            }
        }

        let mut parsed = None;

        if let Some(rlb) = self.rlb.as_mut() {
            while let Line::Some(line) = rlb.next() {
                let s = core::str::from_utf8(line).unwrap_or("");
                let s = s.split('#').next().unwrap_or("").trim();
                if s.is_empty() {
                    continue;
                }

                let mut iter = s.split_whitespace();
                let ip_str = match iter.next() {
                    Some(ip) => ip,
                    None => continue,
                };

                let addr = match inet_aton_inner(ip_str.as_bytes()) {
                    Ok(a) => a,
                    Err(_) => continue,
                };

                let name = match iter.next() {
                    Some(n) => n,
                    None => continue,
                };

                let aliases: Vec<Vec<u8>> = iter
                    .map(|alias| alias.bytes().chain(Some(b'\0')).collect())
                    .collect();

                parsed = Some((addr, name.to_string(), aliases));
                break;
            }
        }

        if let Some((addr, name_str, aliases)) = parsed {
            self.addr_bytes = addr.to_ne_bytes();
            self.addr_list = [self.addr_bytes.as_mut_ptr().cast(), core::ptr::null_mut()];

            self.name = name_str.bytes().chain(Some(b'\0')).collect();
            self.aliases = aliases;
            self.aliases_ptrs = self
                .aliases
                .iter_mut()
                .map(|x| x.as_mut_ptr().cast::<c_char>())
                .chain(Some(core::ptr::null_mut()))
                .collect();

            self.entry = hostent {
                h_name: self.name.as_mut_ptr().cast(),
                h_aliases: self.aliases_ptrs.as_mut_ptr(),
                h_addrtype: AF_INET,
                h_length: 4,
                h_addr_list: self.addr_list.as_mut_ptr(),
            };

            Some(&raw mut self.entry)
        } else {
            if !self.stayopen {
                self.endhostent();
            }
            None
        }
    }

    pub fn gethostbyname_local(&mut self, search_name: &[u8]) -> Option<*mut hostent> {
        self.sethostent_def();

        while let Some(h) = self.gethostent() {
            if self.name.eq_ignore_ascii_case(search_name)
                || self
                    .aliases
                    .iter()
                    .any(|alias| alias.eq_ignore_ascii_case(search_name))
            {
                self.endhostent_def();
                return Some(h);
            }
        }

        self.endhostent_def();
        None
    }

    pub fn gethostbyaddr_local(&mut self, search_addr: u32) -> Option<*mut hostent> {
        self.sethostent_def();

        while let Some(h) = self.gethostent() {
            let current_addr = u32::from_ne_bytes(self.addr_bytes);
            if current_addr == search_addr {
                self.endhostent_def();
                return Some(h);
            }
        }

        self.endhostent_def();
        None
    }
}

unsafe impl Send for HostState {}
pub(crate) static HOST_STATE: Mutex<HostState> = Mutex::new(HostState::new());
