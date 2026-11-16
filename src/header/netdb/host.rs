use alloc::{string::ToString, vec::Vec};

use crate::{
    error::Errno,
    header::{
        arpa_inet::inet_aton_inner, fcntl::O_RDONLY, netdb::hostent, netinet_in::in_addr,
        stdio::SEEK_SET, sys_socket::constants::AF_INET,
    },
    platform::{
        Line, Pal, RawLineBuffer, Sys,
        types::{c_char, c_int},
    },
    sync::Mutex,
};

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
                let Some(ip_str) = iter.next() else {
                    continue;
                };

                let Ok(addr) = inet_aton_inner(ip_str.as_bytes()) else {
                    continue;
                };

                let Some(name) = iter.next() else {
                    continue;
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

    pub fn gethostbyname_external(
        &mut self,
        lookup_host_res: Result<Vec<in_addr>, Errno>,
        search_bytes: &[u8],
    ) -> Result<*mut hostent, c_int> {
        match lookup_host_res {
            Ok(host) => {
                if let Some(host_addr) = host.into_iter().next() {
                    self.name = search_bytes.to_vec();

                    self.addr_bytes = host_addr.s_addr.to_ne_bytes();
                    self.addr_list = [self.addr_bytes.as_mut_ptr().cast(), core::ptr::null_mut()];

                    //TODO actually get aliases
                    self.aliases.clear();
                    self.aliases_ptrs = vec![core::ptr::null_mut(), core::ptr::null_mut()];

                    self.entry = hostent {
                        h_name: self.name.as_mut_ptr().cast(),
                        h_aliases: self.aliases_ptrs.as_mut_ptr(),
                        h_addrtype: AF_INET,
                        h_length: 4,
                        h_addr_list: self.addr_list.as_mut_ptr(),
                    };

                    Ok(&raw mut self.entry)
                } else {
                    Err(super::HOST_NOT_FOUND)
                }
            }
            Err(_) => Err(super::NO_RECOVERY),
        }
    }

    pub fn gethostbyaddr_external(
        &mut self,
        lookup_addr_res: Result<Vec<Vec<u8>>, Errno>,
        addr: in_addr,
    ) -> Result<*mut hostent, c_int> {
        match lookup_addr_res {
            Ok(host_names) => {
                if let Some(host_name) = host_names.into_iter().next() {
                    self.name = host_name;

                    self.addr_bytes = addr.s_addr.to_ne_bytes();
                    self.addr_list = [self.addr_bytes.as_mut_ptr().cast(), core::ptr::null_mut()];

                    //TODO actually get aliases
                    self.aliases.clear();
                    self.aliases_ptrs = vec![core::ptr::null_mut(), core::ptr::null_mut()];

                    self.entry = hostent {
                        h_name: self.name.as_mut_ptr().cast(),
                        h_aliases: self.aliases_ptrs.as_mut_ptr(),
                        h_addrtype: AF_INET,
                        h_length: 4,
                        h_addr_list: self.addr_list.as_mut_ptr(),
                    };

                    Ok(&raw mut self.entry)
                } else {
                    // `glibc` sets errno if an address doesn't have a host name
                    // `musl` uses the address as the host name in said case
                    Err(super::HOST_NOT_FOUND)
                }
            }
            // TODO: Better error separation in lookup_addr
            Err(_) => Err(super::NO_RECOVERY),
        }
    }
}

unsafe impl Send for HostState {}
pub(crate) static HOST_STATE: Mutex<HostState> = Mutex::new(HostState::new());
