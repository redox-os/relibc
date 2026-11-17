use alloc::{string::ToString, vec::Vec};

use crate::{
    header::{fcntl::O_RDONLY, netdb::protoent, stdio::SEEK_SET},
    platform::{
        Line, Pal, RawLineBuffer, Sys,
        types::{c_char, c_int},
    },
    sync::Mutex,
};

pub struct ProtoState {
    db: Option<c_int>,
    rlb: Option<RawLineBuffer>,
    stayopen: bool,

    name: Vec<u8>,
    aliases: Vec<Vec<u8>>,
    aliases_ptrs: Vec<*mut c_char>,
    entry: protoent,
}

impl ProtoState {
    const fn new() -> Self {
        Self {
            db: None,
            rlb: None,
            stayopen: false,
            name: Vec::new(),
            aliases: Vec::new(),
            aliases_ptrs: Vec::new(),
            entry: protoent {
                p_name: core::ptr::null_mut(),
                p_aliases: core::ptr::null_mut(),
                p_proto: 0,
            },
        }
    }

    pub fn endprotoent_def(&mut self) {
        if !self.stayopen {
            self.endprotoent();
        }
    }

    pub fn setprotoent_def(&mut self) {
        self.setprotoent(self.stayopen);
    }

    pub fn endprotoent(&mut self) {
        self.rlb = None;
        if let Some(fd) = self.db.take() {
            let _ = Sys::close(fd);
        }
    }

    pub fn setprotoent(&mut self, stayopen: bool) {
        self.stayopen = stayopen;
        if self.db.is_none() {
            if let Ok(fd) = Sys::open(c"/etc/protocols".into(), O_RDONLY, 0) {
                self.db = Some(fd);
                self.rlb = Some(RawLineBuffer::new(fd));
            }
        } else if let Some(fd) = self.db {
            let _ = Sys::lseek(fd, 0, SEEK_SET);
            self.rlb = Some(RawLineBuffer::new(fd));
        }
    }

    pub fn getprotoent(&mut self) -> Option<*mut protoent> {
        if self.rlb.is_none() {
            if let Ok(fd) = Sys::open(c"/etc/protocols".into(), O_RDONLY, 0) {
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
                let Some(num_str) = iter.next() else {
                    continue;
                };
                let Ok(num) = num_str.parse::<c_int>() else {
                    continue;
                };

                let aliases: Vec<Vec<u8>> = iter
                    .map(|alias| alias.bytes().chain(Some(b'\0')).collect())
                    .collect();

                parsed = Some((name.to_string(), num, aliases));
                break;
            }
        }

        if let Some((name_str, num, aliases)) = parsed {
            self.name = name_str.bytes().chain(Some(b'\0')).collect();
            self.aliases = aliases;
            self.aliases_ptrs = self
                .aliases
                .iter_mut()
                .map(|x| x.as_mut_ptr().cast::<c_char>())
                .chain(Some(core::ptr::null_mut()))
                .collect();

            self.entry = protoent {
                p_name: self.name.as_mut_ptr().cast(),
                p_aliases: self.aliases_ptrs.as_mut_ptr(),
                p_proto: num,
            };

            Some(&raw mut self.entry)
        } else {
            self.endprotoent_def();
            None
        }
    }

    pub fn getprotobynumber(&mut self, number: c_int) -> Option<*mut protoent> {
        self.setprotoent_def();

        while let Some(p) = self.getprotoent() {
            if self.entry.p_proto == number {
                self.endprotoent_def();
                return Some(p);
            }
        }

        self.endprotoent_def();
        None
    }

    pub fn getprotobyname(&mut self, search_name: &[u8]) -> Option<*mut protoent> {
        self.setprotoent_def();

        while let Some(p) = self.getprotoent() {
            if self.name.eq_ignore_ascii_case(search_name)
                || self
                    .aliases
                    .iter()
                    .any(|alias| alias.eq_ignore_ascii_case(search_name))
            {
                self.endprotoent_def();
                return Some(p);
            }
        }

        self.endprotoent_def();
        None
    }
}

unsafe impl Send for ProtoState {}
pub(crate) static PROTO_STATE: Mutex<ProtoState> = Mutex::new(ProtoState::new());
