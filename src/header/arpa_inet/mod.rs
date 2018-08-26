//! arpa/inet implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/arpainet.h.html

use core::str::FromStr;
use core::{mem, ptr, slice, str};

use header::errno::*;
use header::netinet_in::in_addr;
use header::sys_socket;
use platform;
use platform::c_str;
use platform::types::*;

#[no_mangle]
pub extern "C" fn htonl(hostlong: u32) -> u32 {
    hostlong.to_be()
}

#[no_mangle]
pub extern "C" fn htons(hostshort: u16) -> u16 {
    hostshort.to_be()
}

#[no_mangle]
pub extern "C" fn ntohl(netlong: u32) -> u32 {
    u32::from_be(netlong)
}

#[no_mangle]
pub extern "C" fn ntohs(netshort: u16) -> u16 {
    u16::from_be(netshort)
}

static mut NTOA_ADDR: [c_char; 16] = [0; 16];

#[no_mangle]
pub unsafe extern "C" fn inet_aton(cp: *const c_char, inp: *mut in_addr) -> c_int {
    // TODO: octal/hex
    inet_pton(AF_INET, cp, inp as *mut c_void)
}

#[no_mangle]
pub unsafe extern "C" fn inet_ntoa(addr: in_addr) -> *const c_char {
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
        let mut s_addr = slice::from_raw_parts_mut(
            &mut (*(dest as *mut in_addr)).s_addr as *mut _ as *mut u8,
            4,
        );
        let mut octets = str::from_utf8_unchecked(c_str(src)).split('.');
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
pub extern "C" fn inet_addr(cp: *const c_char) -> in_addr_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn inet_lnaof(_in: in_addr) -> in_addr_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn inet_makeaddr(net: in_addr_t, lna: in_addr_t) -> in_addr {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn inet_netof(_in: in_addr) -> in_addr_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn inet_network(cp: *const c_char) -> in_addr_t {
    unimplemented!();
}
