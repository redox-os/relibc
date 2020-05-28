//! arpa/inet implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xns/arpainet.h.html

use core::{
    ptr, slice,
    str::{self, FromStr},
};

use crate::{
    c_str::CStr,
    header::{
        errno::*,
        netinet_in::{in_addr, in_addr_t, INADDR_NONE},
        sys_socket::{constants::*, socklen_t},
    },
    platform::{self, types::*},
};

#[no_mangle]
pub extern "C" fn htonl(hostlong: uint32_t) -> uint32_t {
    hostlong.to_be()
}

#[no_mangle]
pub extern "C" fn htons(hostshort: uint16_t) -> uint16_t {
    hostshort.to_be()
}

#[no_mangle]
pub extern "C" fn ntohl(netlong: uint32_t) -> uint32_t {
    u32::from_be(netlong)
}

#[no_mangle]
pub extern "C" fn ntohs(netshort: uint16_t) -> uint16_t {
    u16::from_be(netshort)
}

#[no_mangle]
pub unsafe extern "C" fn inet_aton(cp: *const c_char, inp: *mut in_addr) -> c_int {
    // TODO: octal/hex
    inet_pton(AF_INET, cp, inp as *mut c_void)
}

#[no_mangle]
pub unsafe extern "C" fn inet_ntoa(addr: in_addr) -> *const c_char {
    static mut NTOA_ADDR: [c_char; 16] = [0; 16];

    inet_ntop(
        AF_INET,
        &addr as *const in_addr as *const c_void,
        NTOA_ADDR.as_mut_ptr(),
        16,
    )
}

#[no_mangle]
pub unsafe extern "C" fn inet_pton(domain: c_int, src: *const c_char, dest: *mut c_void) -> c_int {
    if domain != AF_INET {
        platform::errno = EAFNOSUPPORT;
        -1
    } else {
        let s_addr = slice::from_raw_parts_mut(
            &mut (*(dest as *mut in_addr)).s_addr as *mut _ as *mut u8,
            4,
        );
        let src_cstr = CStr::from_ptr(src);
        let mut octets = str::from_utf8_unchecked(src_cstr.to_bytes()).split('.');
        for i in 0..4 {
            if let Some(n) = octets.next().and_then(|x| u8::from_str(x).ok()) {
                s_addr[i] = n;
            } else {
                return 0;
            }
        }
        if octets.next() == None {
            1 // Success
        } else {
            0
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn inet_ntop(
    domain: c_int,
    src: *const c_void,
    dest: *mut c_char,
    size: socklen_t,
) -> *const c_char {
    if domain != AF_INET {
        platform::errno = EAFNOSUPPORT;
        ptr::null()
    } else if size < 16 {
        platform::errno = ENOSPC;
        ptr::null()
    } else {
        let s_addr = slice::from_raw_parts(
            &(*(src as *const in_addr)).s_addr as *const _ as *const u8,
            4,
        );
        let addr = format!("{}.{}.{}.{}\0", s_addr[0], s_addr[1], s_addr[2], s_addr[3]);
        ptr::copy(addr.as_ptr() as *const c_char, dest, addr.len());
        dest
    }
}

#[no_mangle]
pub unsafe extern "C" fn inet_addr(cp: *const c_char) -> in_addr_t {
    let mut val: in_addr = in_addr { s_addr: 0 };

    if inet_aton(cp, &mut val) > 0 {
        val.s_addr
    } else {
        INADDR_NONE
    }
}

#[no_mangle]
pub extern "C" fn inet_lnaof(input: in_addr) -> in_addr_t {
    if input.s_addr >> 24 < 128 {
        input.s_addr & 0xff_ffff
    } else if input.s_addr >> 24 < 192 {
        input.s_addr & 0xffff
    } else {
        input.s_addr & 0xff
    }
}

#[no_mangle]
pub extern "C" fn inet_makeaddr(net: in_addr_t, host: in_addr_t) -> in_addr {
    let mut output: in_addr = in_addr { s_addr: 0 };

    if net < 256 {
        output.s_addr = host | net << 24;
    } else if net < 65536 {
        output.s_addr = host | net << 16;
    } else {
        output.s_addr = host | net << 8;
    }

    output
}

#[no_mangle]
pub extern "C" fn inet_netof(input: in_addr) -> in_addr_t {
    if input.s_addr >> 24 < 128 {
        input.s_addr & 0xff_ffff
    } else if input.s_addr >> 24 < 192 {
        input.s_addr & 0xffff
    } else {
        input.s_addr & 0xff
    }
}

#[no_mangle]
pub unsafe extern "C" fn inet_network(cp: *mut c_char) -> in_addr_t {
    ntohl(inet_addr(cp))
}
