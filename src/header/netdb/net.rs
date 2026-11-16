use alloc::{string::ToString, vec::Vec};

use crate::{
    header::{
        arpa_inet::inet_aton_inner, bits_arpainet::ntohl, fcntl::O_RDONLY, netdb::netent,
        stdio::SEEK_SET, sys_socket::constants::AF_INET,
    },
    platform::{
        Line, Pal, RawLineBuffer, Sys,
        types::{c_char, c_int},
    },
    sync::Mutex,
};

#[derive(Default)]
pub struct NetState {
    db: Option<c_int>,
    rlb: Option<RawLineBuffer>,
    stayopen: bool,

    name: Vec<u8>,
    aliases: Vec<Vec<u8>>,
    aliases_ptrs: Vec<*mut c_char>,
    entry: netent,
}

impl NetState {
    const fn new() -> Self {
        Self {
            db: None,
            rlb: None,
            stayopen: false,
            name: Vec::new(),
            aliases: Vec::new(),
            aliases_ptrs: Vec::new(),
            entry: netent {
                n_name: core::ptr::null_mut(),
                n_aliases: core::ptr::null_mut(),
                n_addrtype: 0,
                n_net: 0,
            },
        }
    }

    pub fn endnetent_def(&mut self) {
        if !self.stayopen {
            self.endnetent();
        }
    }

    pub fn setnetent_def(&mut self) {
        self.setnetent(self.stayopen);
    }

    pub fn endnetent(&mut self) {
        self.rlb = None;
        if let Some(fd) = self.db.take() {
            let _ = Sys::close(fd);
        }
    }

    pub fn setnetent(&mut self, stayopen: bool) {
        self.stayopen = stayopen;
        if self.db.is_none() {
            if let Ok(fd) = Sys::open(c"/etc/networks".into(), O_RDONLY, 0) {
                self.db = Some(fd);
                self.rlb = Some(RawLineBuffer::new(fd));
            }
        } else if let Some(fd) = self.db {
            let _ = Sys::lseek(fd, 0, SEEK_SET);
            self.rlb = Some(RawLineBuffer::new(fd));
        }
    }

    pub fn getnetent(&mut self) -> Option<*mut netent> {
        if self.rlb.is_none() {
            if let Ok(fd) = Sys::open(c"/etc/networks".into(), O_RDONLY, 0) {
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
                let Some(name) = iter.next() else {
                    continue;
                };
                let Some(addr_str) = iter.next() else {
                    continue;
                };
                let Ok(addr) = inet_aton_inner(addr_str.as_bytes()) else {
                    continue;
                };

                let aliases: Vec<Vec<u8>> = iter
                    .map(|alias| alias.bytes().chain(Some(b'\0')).collect())
                    .collect();

                parsed = Some((name.to_string(), addr, aliases));
                break;
            }
        }

        if let Some((name_str, addr, aliases)) = parsed {
            self.name = name_str.bytes().chain(Some(b'\0')).collect();
            self.aliases = aliases;
            self.aliases_ptrs = self
                .aliases
                .iter_mut()
                .map(|x| x.as_mut_ptr().cast::<c_char>())
                .chain(Some(core::ptr::null_mut()))
                .collect();

            self.entry = netent {
                n_name: self.name.as_mut_ptr().cast(),
                n_aliases: self.aliases_ptrs.as_mut_ptr(),
                n_addrtype: AF_INET,
                n_net: ntohl(addr),
            };

            Some(&raw mut self.entry)
        } else {
            self.endnetent_def();
            None
        }
    }

    pub fn getnetbyaddr(&mut self, net: u32, net_type: c_int) -> Option<*mut netent> {
        if net_type != AF_INET {
            return None;
        }

        self.setnetent_def();

        while let Some(n) = self.getnetent() {
            if self.entry.n_net == net {
                self.endnetent_def();
                return Some(n);
            }
        }

        self.endnetent_def();
        None
    }

    pub fn getnetbyname(&mut self, search_name: &[u8]) -> Option<*mut netent> {
        self.setnetent_def();

        while let Some(n) = self.getnetent() {
            if self.name.eq_ignore_ascii_case(search_name)
                || self
                    .aliases
                    .iter()
                    .any(|alias| alias.eq_ignore_ascii_case(search_name))
            {
                self.endnetent_def();
                return Some(n);
            }
        }

        self.endnetent_def();
        None
    }
}

unsafe impl Send for NetState {}
pub(crate) static NET_STATE: Mutex<NetState> = Mutex::new(NetState::new());
