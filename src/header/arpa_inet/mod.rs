//! `arpa/inet.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/arpa_inet.h.html>.

// TODO: set this for entire crate when possible
#![deny(unsafe_op_in_unsafe_fn)]

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

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/htonl.html>.
#[no_mangle]
pub extern "C" fn htonl(hostlong: uint32_t) -> uint32_t {
    hostlong.to_be()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/htonl.html>.
#[no_mangle]
pub extern "C" fn htons(hostshort: uint16_t) -> uint16_t {
    hostshort.to_be()
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/inet_addr.html>.
///
/// # Deprecated
/// The `inet_addr()` function was marked obsolescent in the Open Group Base
/// Specifications Issue 8.
#[deprecated]
#[no_mangle]
pub unsafe extern "C" fn inet_addr(cp: *const c_char) -> in_addr_t {
    let mut val: in_addr = in_addr { s_addr: 0 };

    if unsafe { inet_aton(cp, &mut val) } > 0 {
        val.s_addr
    } else {
        INADDR_NONE
    }
}

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/inet_aton.3.html>.
#[no_mangle]
pub unsafe extern "C" fn inet_aton(cp: *const c_char, inp: *mut in_addr) -> c_int {
    // TODO: octal/hex
    unsafe { inet_pton(AF_INET, cp, inp as *mut c_void) }
}

/// See <https://pubs.opengroup.org/onlinepubs/7908799/xns/inet_lnaof.html>.
///
/// # Deprecation
/// The `inet_lnaof()` function was specified in Networking Services Issue 5,
/// but not in the Open Group Base Specifications Issue 6 and later.
#[deprecated]
#[no_mangle]
pub extern "C" fn inet_lnaof(r#in: in_addr) -> in_addr_t {
    if r#in.s_addr >> 24 < 128 {
        r#in.s_addr & 0xff_ffff
    } else if r#in.s_addr >> 24 < 192 {
        r#in.s_addr & 0xffff
    } else {
        r#in.s_addr & 0xff
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/7908799/xns/inet_makeaddr.html>.
///
/// # Deprecation
/// The `inet_makeaddr()` function was specified in Networking Services Issue
/// 5, but not in the Open Group Base Specifications Issue 6 and later.
#[deprecated]
#[no_mangle]
pub extern "C" fn inet_makeaddr(net: in_addr_t, lna: in_addr_t) -> in_addr {
    let mut output: in_addr = in_addr { s_addr: 0 };

    if net < 256 {
        output.s_addr = lna | net << 24;
    } else if net < 65536 {
        output.s_addr = lna | net << 16;
    } else {
        output.s_addr = lna | net << 8;
    }

    output
}

/// See <https://pubs.opengroup.org/onlinepubs/7908799/xns/inet_netof.html>.
///
/// # Deprecation
/// The `inet_netof()` function was specified in Networking Services Issue 5,
/// but not in the Open Group Base Specifications Issue 6 and later.
#[deprecated]
#[no_mangle]
pub extern "C" fn inet_netof(r#in: in_addr) -> in_addr_t {
    if r#in.s_addr >> 24 < 128 {
        r#in.s_addr & 0xff_ffff
    } else if r#in.s_addr >> 24 < 192 {
        r#in.s_addr & 0xffff
    } else {
        r#in.s_addr & 0xff
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/7908799/xns/inet_network.html>.
///
/// # Deprecation
/// The `inet_network()` function was specified in Networking Services Issue 5,
/// but not in the Open Group Base Specifications Issue 6 and later.
#[deprecated]
#[no_mangle]
pub unsafe extern "C" fn inet_network(cp: *const c_char) -> in_addr_t {
    ntohl(unsafe { inet_addr(cp) })
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/inet_addr.html>.
///
/// # Deprecation
/// The `inet_ntoa()` function was marked obsolescent in the Open Group Base
/// Specifications Issue 8.
#[deprecated]
#[no_mangle]
pub unsafe extern "C" fn inet_ntoa(r#in: in_addr) -> *const c_char {
    static mut NTOA_ADDR: [c_char; 16] = [0; 16];

    unsafe {
        inet_ntop(
            AF_INET,
            &r#in as *const in_addr as *const c_void,
            NTOA_ADDR.as_mut_ptr(),
            16,
        )
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/inet_ntop.html>.
#[no_mangle]
pub unsafe extern "C" fn inet_ntop(
    af: c_int,
    src: *const c_void,
    dst: *mut c_char,
    size: socklen_t,
) -> *const c_char {
    if af != AF_INET {
        platform::ERRNO.set(EAFNOSUPPORT);
        ptr::null()
    } else if size < 16 {
        platform::ERRNO.set(ENOSPC);
        ptr::null()
    } else {
        let s_addr = unsafe {
            slice::from_raw_parts(
                &(*(src as *const in_addr)).s_addr as *const _ as *const u8,
                4,
            )
        };
        let addr = format!("{}.{}.{}.{}\0", s_addr[0], s_addr[1], s_addr[2], s_addr[3]);
        unsafe {
            ptr::copy(addr.as_ptr() as *const c_char, dst, addr.len());
        }
        dst
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/inet_ntop.html>.
#[no_mangle]
pub unsafe extern "C" fn inet_pton(af: c_int, src: *const c_char, dst: *mut c_void) -> c_int {
    if af != AF_INET {
        platform::ERRNO.set(EAFNOSUPPORT);
        -1
    } else {
        let s_addr = unsafe {
            slice::from_raw_parts_mut(
                &mut (*(dst as *mut in_addr)).s_addr as *mut _ as *mut u8,
                4,
            )
        };
        let src_cstr = unsafe { CStr::from_ptr(src) };
        let mut octets = unsafe { str::from_utf8_unchecked(src_cstr.to_bytes()).split('.') };
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

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/htonl.html>.
#[no_mangle]
pub extern "C" fn ntohl(netlong: uint32_t) -> uint32_t {
    u32::from_be(netlong)
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/htonl.html>.
#[no_mangle]
pub extern "C" fn ntohs(netshort: uint16_t) -> uint16_t {
    u16::from_be(netshort)
}
