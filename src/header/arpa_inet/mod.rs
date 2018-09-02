//! arpa/inet implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xns/arpainet.h.html

use core::str::FromStr;
use core::{ptr, slice, str};

use header::errno::*;
use header::netinet_in::{in_addr, in_addr_t};
use header::ctype::{isdigit, isxdigit, tolower, isspace};
use platform;
use platform::c_str;
use platform::types::*;


const IN_CLASSA_NET: u32 = 0xff000000;
const IN_CLASSA_NSHIFT: u32 = 24;
const IN_CLASSA_MAX: u32 = 128;
const IN_CLASSB_NET: u32 = 0xffff0000;
const IN_CLASSB_NSHIFT: u32 = 16;
const IN_CLASSB_MAX: u32 = 65536;
const IN_CLASSC_NET: u32 = 0xffffff00;
const IN_CLASSC_NSHIFT: u32 = 8;
const INADDR_NONE: u32 = 0xffffffff;

fn IN_CLASSA(a: in_addr_t) -> bool {
    a & 0x80000000 == 0
}

fn IN_CLASSB(a: in_addr_t) -> bool {
    a & 0xc0000000 == 0x80000000
}

fn IN_CLASSC(a: in_addr_t) -> bool {
    a & 0xe0000000 == 0xc0000000
}

fn IN_CLASSD(a: in_addr_t) -> bool {
    a & 0xf0000000 == 0xe0000000
}

fn IN_CLASSA_HOST() -> u32 {
    0xffffffff & !IN_CLASSA_NET
}

fn IN_CLASSB_HOST() -> u32 {
    0xffffffff & !IN_CLASSB_NET
}

fn IN_CLASSC_HOST() -> u32 {
    0xffffffff & !IN_CLASSC_NET
}


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
    let mut val: in_addr = in_addr {
        s_addr: 0,
    };

    if unsafe { inet_aton(cp, &mut val) } {
        return val.s_addr;
    }

    INADDR_NONE
}

#[no_mangle]
pub extern "C" fn inet_lnaof(_in: in_addr) -> in_addr_t {
    let i: u32 = ntohl(_in.s_addr);

    if IN_CLASSA(i) {
        i & IN_CLASSA_HOST()
    } else if IN_CLASSB(i) {
        i & IN_CLASSB_HOST()
    } else {
        i & IN_CLASSC_HOST()
    }
}

#[no_mangle]
pub extern "C" fn inet_makeaddr(net: in_addr_t, host: in_addr_t) -> in_addr {
    let mut input: in_addr = in_addr {
        s_addr: 0,
    };

    if net < 128 {
        input.s_addr = (net << IN_CLASSA_NSHIFT) | (host & IN_CLASSA_HOST());
    } else if net < 65536 {
        input.s_addr = (net << IN_CLASSB_NSHIFT) | (host & IN_CLASSB_HOST());
    } else if net < 16777216 {
        input.s_addr = (net << IN_CLASSC_NSHIFT) | (host & IN_CLASSC_HOST());
    } else {
        input.s_addr = net | host;
    }

    input.s_addr = htonl(input.s_addr);
    return input;
}

#[no_mangle]
pub extern "C" fn inet_netof(_in: in_addr) -> in_addr_t {
    let i: u32 = ntohl(_in.s_addr);
    if IN_CLASSA(i) {
        ( i & IN_CLASSA_NET ) >> IN_CLASSA_NSHIFT
    } else if IN_CLASSB(i) {
        ( i & IN_CLASSB_NET ) >> IN_CLASSB_NSHIFT
    } else {
        ( i & IN_CLASSC_NET ) >> IN_CLASSC_NSHIFT
    }
}



#[no_mangle]
pub extern "C" fn inet_network(cp: *mut c_char) -> in_addr_t {
    let mut val: u32;
    let mut base: u32;
    let mut n: u32;
    let i: u32;
    let c: char;
    let mut parts: [u32; 4];
    let mut pp: *mut u32 = parts.as_mut_ptr();
    let mut digit: i32;

    #[derive(PartialEq)]
    enum Loop {
        Continue,
        Stop,
    }

    let mut loop_state: Loop = Loop::Stop;




    while loop_state == Loop::Continue {
        val = 0;
        base = 10;
        digit = 0;

        if unsafe  { *cp == 0 } {
            digit = 1;
            base = 8;
            unsafe { cp.offset(1) };
        }

        if unsafe { *cp as u8 as char == 'x' || *cp as u8 as char == 'X' } {
            digit = 0;
            base = 16;
            unsafe { cp.offset(1) };
        }


        while (c = unsafe { *cp }) != 0 {
            if isdigit(c) {
                if base == 8 && (c == '8' || c == '9') {
                    return INADDR_NONE;
                }
                val = (val * base) + (c - '0');
                unsafe { cp.offset(1) };
                digit = 1;
                continue;
            }

            if base == 16 && isxdigit(c) {
                val = (val << 4) + (tolower (c) + 10 - 'a');
                unsafe { cp.offset(1) };
                digit = 1;
                continue;
            }
            break;
        }

        if !digit {
            return INADDR_NONE;
        }

        if pp >= parts + 4 || val > 0xff {
            return INADDR_NONE;
        }

        if unsafe { *cp as u8 as char } == '.' {
            pp = unsafe { pp.offset(1) };
            unsafe { *pp = val };
            unsafe { cp.offset(1) };
            loop_state = Loop::Continue;
        } else {
            loop_state = Loop::Stop;
        }

        while isspace(unsafe { *cp as i32 }) {
            unsafe { cp.offset(1) };
        }

        if unsafe { *cp } {
            return INADDR_NONE;
        }

        if pp >= parts + 4 || val > 0xff {
            return INADDR_NONE;
        }


        pp = unsafe { pp.offset(1) };
        unsafe { *pp = val };

        n = pp - parts;

        for (mut val, i) in i < n {
            val <<= 8;
            val |= parts[i] & 0xff;
        }
    }
    val
}
