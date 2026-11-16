use alloc::{string::ToString, vec::Vec};

use crate::{
    header::{bits_arpainet::htons, fcntl::O_RDONLY, netdb::servent, stdio::SEEK_SET},
    platform::{
        Line, Pal, RawLineBuffer, Sys,
        types::{c_char, c_int},
    },
    sync::Mutex,
};

pub struct ServState {
    db: Option<c_int>,
    rlb: Option<RawLineBuffer>,
    stayopen: bool,

    name: Vec<u8>,
    proto: Vec<u8>,
    aliases: Vec<Vec<u8>>,
    aliases_ptrs: Vec<*mut c_char>,
    entry: servent,
}

impl ServState {
    const fn new() -> Self {
        Self {
            db: None,
            rlb: None,
            stayopen: false,
            name: Vec::new(),
            proto: Vec::new(),
            aliases: Vec::new(),
            aliases_ptrs: Vec::new(),
            entry: servent {
                s_name: core::ptr::null_mut(),
                s_aliases: core::ptr::null_mut(),
                s_port: 0,
                s_proto: core::ptr::null_mut(),
            },
        }
    }

    pub fn endservent_def(&mut self) {
        if !self.stayopen {
            self.endservent();
        }
    }

    pub fn setservent_def(&mut self) {
        self.setservent(self.stayopen);
    }

    pub fn endservent(&mut self) {
        self.rlb = None;
        if let Some(fd) = self.db.take() {
            let _ = Sys::close(fd);
        }
    }

    pub fn setservent(&mut self, stayopen: bool) {
        self.stayopen = stayopen;
        if self.db.is_none() {
            if let Ok(fd) = Sys::open(c"/etc/services".into(), O_RDONLY, 0) {
                self.db = Some(fd);
                self.rlb = Some(RawLineBuffer::new(fd));
            }
        } else if let Some(fd) = self.db {
            let _ = Sys::lseek(fd, 0, SEEK_SET);
            self.rlb = Some(RawLineBuffer::new(fd));
        }
    }

    pub fn getservent(&mut self) -> Option<*mut servent> {
        if self.rlb.is_none() {
            if let Ok(fd) = Sys::open(c"/etc/services".into(), O_RDONLY, 0) {
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
                let Some(port_proto) = iter.next() else {
                    continue;
                };

                let mut split = port_proto.split('/');
                let Some(port_u16) = split.next().and_then(|p| p.parse::<u16>().ok()) else {
                    continue;
                };
                let Some(proto) = split.next() else {
                    continue;
                };

                let aliases: Vec<Vec<u8>> = iter
                    .map(|alias| alias.bytes().chain(Some(b'\0')).collect())
                    .collect();

                parsed = Some((name.to_string(), port_u16, proto.to_string(), aliases));
                break;
            }
        }

        if let Some((name_str, port_u16, proto_str, aliases)) = parsed {
            self.name = name_str.bytes().chain(Some(b'\0')).collect();
            self.proto = proto_str.bytes().chain(Some(b'\0')).collect();
            self.aliases = aliases;
            self.aliases_ptrs = self
                .aliases
                .iter_mut()
                .map(|x| x.as_mut_ptr().cast::<c_char>())
                .chain(Some(core::ptr::null_mut()))
                .collect();

            self.entry = servent {
                s_name: self.name.as_mut_ptr().cast(),
                s_aliases: self.aliases_ptrs.as_mut_ptr(),
                s_port: c_int::from(htons(port_u16)),
                s_proto: self.proto.as_mut_ptr().cast(),
            };

            Some(&raw mut self.entry)
        } else {
            if !self.stayopen {
                self.endservent();
            }
            None
        }
    }

    pub fn getservbyport(
        &mut self,
        port: c_int,
        search_proto: Option<&[u8]>,
    ) -> Option<*mut servent> {
        self.setservent_def();

        while let Some(s) = self.getservent() {
            if self.entry.s_port != port {
                continue;
            }
            if search_proto.is_none_or(|p| self.proto.eq_ignore_ascii_case(p)) {
                self.endservent_def();
                return Some(s);
            }
        }

        self.endservent_def();
        None
    }

    pub fn getservbyname(
        &mut self,
        search_name: &[u8],
        search_proto: Option<&[u8]>,
    ) -> Option<*mut servent> {
        self.setservent_def();

        while let Some(s) = self.getservent() {
            if (self.name.eq_ignore_ascii_case(search_name)
                || self
                    .aliases
                    .iter()
                    .any(|alias| alias.eq_ignore_ascii_case(search_name)))
                && search_proto.is_none_or(|p| self.proto.eq_ignore_ascii_case(p))
            {
                self.endservent_def();
                return Some(s);
            }
        }

        self.endservent_def();
        None
    }
}

unsafe impl Send for ServState {}
pub(crate) static SERV_STATE: Mutex<ServState> = Mutex::new(ServState::new());
