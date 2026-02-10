use alloc::{boxed::Box, str::SplitWhitespace, vec::Vec};
use core::{mem, ptr};

use crate::{
    error::ResultExt,
    header::{
        arpa_inet::inet_aton, fcntl::O_RDONLY, netinet_in::in_addr, sys_socket::constants::AF_INET,
        unistd::SEEK_SET,
    },
    platform::{
        Pal, Sys,
        rlb::{Line, RawLineBuffer},
        types::{c_char, c_int},
    },
    raw_cell::RawCell,
};

use super::{bytes_to_box_str, hostent};

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

#[unsafe(no_mangle)]
pub unsafe extern "C" fn endhostent() {
    if unsafe { HOSTDB } >= 0 {
        Sys::close(unsafe { HOSTDB });
    }
    unsafe { HOSTDB = -1 };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn sethostent(stayopen: c_int) {
    unsafe { HOST_STAYOPEN = stayopen };
    if unsafe { HOSTDB } < 0 {
        unsafe { HOSTDB = Sys::open(c"/etc/hosts".into(), O_RDONLY, 0).or_minus_one_errno() }
    } else {
        Sys::lseek(unsafe { HOSTDB }, 0, SEEK_SET);
    }
    unsafe { H_POS = 0 };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn gethostent() -> *mut hostent {
    if unsafe { HOSTDB } < 0 {
        unsafe { HOSTDB = Sys::open(c"/etc/hosts".into(), O_RDONLY, 0).or_minus_one_errno() };
    }
    let mut rlb = RawLineBuffer::new(unsafe { HOSTDB });
    rlb.seek(unsafe { H_POS });

    let mut r: Box<str> = Box::default();
    while r.is_empty() || r.split_whitespace().next() == None || r.starts_with('#') {
        r = match rlb.next() {
            Line::Some(s) => bytes_to_box_str(s),
            _ => {
                if unsafe { HOST_STAYOPEN } == 0 {
                    unsafe { endhostent() };
                }
                return ptr::null_mut();
            }
        };
    }
    rlb.next();
    unsafe { H_POS = rlb.line_pos() };

    let mut iter: SplitWhitespace = r.split_whitespace();

    let addr_vec: Vec<u8> = iter.next().unwrap().bytes().chain(Some(b'\0')).collect();
    let addr_cstr = addr_vec.as_slice().as_ptr() as *const c_char;
    let mut addr = mem::MaybeUninit::uninit();
    unsafe { inet_aton(addr_cstr, addr.as_mut_ptr()) };
    let addr = unsafe { addr.assume_init() };

    unsafe {
        _HOST_ADDR_LIST = addr.s_addr.to_ne_bytes();
        HOST_ADDR_LIST = [&raw mut _HOST_ADDR_LIST as *mut c_char, ptr::null_mut()];

        HOST_ADDR = Some(addr);
    }

    let host_name = iter.next().unwrap().bytes().chain(Some(b'\0')).collect();

    let mut _host_aliases: Vec<Vec<u8>> = iter
        .map(|alias| alias.bytes().chain(Some(b'\0')).collect())
        .collect();
    unsafe { HOST_ALIASES.unsafe_set(Some(_host_aliases)) };

    let mut host_aliases: Vec<*mut c_char> = unsafe {
        HOST_ALIASES
            .unsafe_mut()
            .as_mut()
            .unwrap()
            .iter_mut()
            .map(|x| x.as_mut_ptr() as *mut c_char)
            .chain([ptr::null_mut(), ptr::null_mut()])
            .collect()
    };

    unsafe { HOST_NAME.unsafe_set(Some(host_name)) };

    unsafe {
        HOST_ENTRY = hostent {
            h_name: HOST_NAME.unsafe_mut().as_mut().unwrap().as_mut_ptr() as *mut c_char,
            h_aliases: host_aliases.as_mut_slice().as_mut_ptr(),
            h_addrtype: AF_INET,
            h_length: 4,
            h_addr_list: &raw mut HOST_ADDR_LIST as *mut _,
        };
        _HOST_ALIASES.unsafe_set(Some(host_aliases));
    }
    if unsafe { HOST_STAYOPEN } == 0 {
        unsafe { endhostent() };
    }
    &raw mut HOST_ENTRY as *mut hostent
}
