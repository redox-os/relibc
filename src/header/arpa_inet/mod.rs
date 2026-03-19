//! `arpa/inet.h` implementation.
//!
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/arpa_inet.h.html>.

use core::{
    ptr, slice,
    str::{self, FromStr},
};

use alloc::string::ToString;

use crate::{
    c_str::CStr,
    header::{
        bits_socklen_t::socklen_t,
        errno::{EAFNOSUPPORT, ENOSPC},
        netinet_in::{INADDR_NONE, in_addr, in_addr_t, ntohl},
        sys_socket::constants::AF_INET,
    },
    platform::{
        self,
        types::{c_char, c_int, c_void},
    },
    raw_cell::RawCell,
};

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/inet_addr.html>.
///
/// # Deprecated
/// The `inet_addr()` function was marked obsolescent in the Open Group Base
/// Specifications Issue 8.
#[deprecated]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn inet_addr(cp: *const c_char) -> in_addr_t {
    let mut val: in_addr = in_addr { s_addr: 0 };

    if unsafe { inet_aton(cp, &raw mut val) } > 0 {
        val.s_addr
    } else {
        INADDR_NONE
    }
}

/// Non-POSIX, see <https://www.man7.org/linux/man-pages/man3/inet_aton.3.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn inet_aton(cp: *const c_char, inp: *mut in_addr) -> c_int {
    let cp_cstr = unsafe { CStr::from_ptr(cp) };
    if cp_cstr.contains(b'.') {
        // 2, 3 or 4 part address
        let mut parts = unsafe { str::from_utf8_unchecked(cp_cstr.to_bytes()).split('.') };
        if let (_, Some(amount)) = parts.size_hint()
            && let Some(first) = parts.next()
            // 1st part always represents a single u8 (leftmost byte)
            && let Some((first_num, first_notation)) = part_to_num_with_notation(first)
            // 2nd part guaranteed to be present
            && let Some(second) = parts.next()
        {
            match amount {
                // 2nd part = 24bit value defines 3 rightmost bytes
                2 => {
                    todo_skip!(0, "parsing 24bit value as 3 rightmost bytes unimplemented");
                    0 // TODO: remove and implement above
                }
                // 2nd part = 2nd byte, 3rd part = 16bit value defines 2 rightmost bytes
                3 => {
                    todo_skip!(0, "parsing 16bit value as 2 rightmost bytes unimplemented");
                    0 // TODO: remove and implement above
                }
                // each part = 1 byte of address
                4 => {
                    if let Some((second_num, second_notation)) = part_to_num_with_notation(second)
                        && let Some(third) = parts.next()
                        && let Some((third_num, third_notation)) = part_to_num_with_notation(third)
                        && let Some(fourth) = parts.next()
                        && let Some((fourth_num, fourth_notation)) =
                            part_to_num_with_notation(fourth)
                    {
                        match (
                            first_notation,
                            second_notation,
                            third_notation,
                            fourth_notation,
                        ) {
                            (
                                NumNotation::Decimal,
                                NumNotation::Decimal,
                                NumNotation::Decimal,
                                NumNotation::Decimal,
                            ) => unsafe { inet_pton(AF_INET, cp, inp.cast::<c_void>()) },
                            _ => {
                                let mut all = first_num.to_string();
                                all.push('.');
                                all.push_str(&second_num.to_string());
                                all.push('.');
                                all.push_str(&third_num.to_string());
                                all.push('.');
                                all.push_str(&fourth_num.to_string());
                                all.push('\0');
                                let all = all.into_bytes();
                                let new_cp_cstr =
                                    unsafe { CStr::from_bytes_with_nul_unchecked(&all) };
                                unsafe {
                                    inet_pton(AF_INET, new_cp_cstr.as_ptr(), inp.cast::<c_void>())
                                }
                            }
                        }
                    } else {
                        0 // indicates `cp` is an invalid string
                    }
                }
                _ => 0, // indicates `cp` is an invalid string
            }
        } else {
            0 // indicates `cp` is an invalid string
        }
    } else if cp_cstr.len() == 4 {
        // 1 part address (32 bit value to be stored directly into address without byte rearrangement)
        let s_addr_bytes: [u8; 4] = cp_cstr.to_bytes().try_into().expect("guaranteed 4 bytes");
        unsafe {
            (*inp).s_addr = in_addr_t::from_ne_bytes(s_addr_bytes);
        }
        1 // successful
    } else {
        0 // indicates `cp` is an invalid string
    }
}

enum NumNotation {
    Octal,
    Decimal,
    Hexadecimal,
}

// Parses the input into u8 but indicates the original notation
// Returns None for parsing failure
fn part_to_num_with_notation(input: &str) -> Option<(u8, NumNotation)> {
    if let Some(hex_or_oct) = input.strip_prefix('0')
        && input.len() > 1
    {
        let mut num = 0;
        match hex_or_oct.bytes().next() {
            Some(b'x' | b'X') => {
                let (_, hex) = input.split_at(2);
                let bytes = hex.bytes().rev();
                for (i, byte) in bytes.enumerate() {
                    let i = u8::try_from(i).expect("never more than 3 digits");
                    let byte = match byte {
                        b'f' | b'F' => 15,
                        b'e' | b'E' => 14,
                        b'd' | b'D' => 13,
                        b'c' | b'C' => 12,
                        b'b' | b'B' => 11,
                        b'a' | b'A' => 10,
                        _ => str::from_utf8(&[byte])
                            .expect("already checked")
                            .parse::<u8>()
                            .expect("only numbers possible"),
                    };
                    num += if i == 0 { byte } else { byte * (16 * i) };
                }
                Some((num, NumNotation::Hexadecimal))
            }
            // TODO: C2Y accept `0o` or `0O` as octal prefixes, C23 and below only use `0`
            _ => {
                let bytes = hex_or_oct.bytes().rev();
                for (i, byte) in bytes.enumerate() {
                    let i = u8::try_from(i).expect("never more than 3 digits");
                    let byte = str::from_utf8(&[byte])
                        .expect("already checked")
                        .parse::<u8>()
                        .expect("octal always within 0 to 7");
                    num += if i == 0 { byte } else { byte * (8 * i) };
                }
                Some((num, NumNotation::Octal))
            }
        }
    } else if let Ok(num) = input.parse::<u8>() {
        Some((num, NumNotation::Decimal))
    } else {
        None
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/7908799/xns/inet_lnaof.html>.
///
/// # Deprecation
/// The `inet_lnaof()` function was specified in Networking Services Issue 5,
/// but not in the Open Group Base Specifications Issue 6 and later.
#[deprecated]
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn inet_network(cp: *const c_char) -> in_addr_t {
    ntohl(unsafe { inet_addr(cp) })
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/inet_addr.html>.
///
/// # Deprecation
/// The `inet_ntoa()` function was marked obsolescent in the Open Group Base
/// Specifications Issue 8.
#[deprecated]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn inet_ntoa(r#in: in_addr) -> *mut c_char {
    static NTOA_ADDR: RawCell<[c_char; 16]> = RawCell::new([0; 16]);

    unsafe {
        let ptr = inet_ntop(
            AF_INET,
            ptr::from_ref::<in_addr>(&r#in).cast::<c_void>(),
            NTOA_ADDR.unsafe_mut().as_mut_ptr(),
            NTOA_ADDR.unsafe_ref().len() as socklen_t,
        );
        // Mutable pointer is required, inet_ntop returns destination as const pointer
        ptr.cast_mut()
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/inet_ntop.html>.
#[unsafe(no_mangle)]
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
                ptr::from_ref(&(*(src.cast::<in_addr>())).s_addr).cast::<u8>(),
                4,
            )
        };
        let addr = format!("{}.{}.{}.{}\0", s_addr[0], s_addr[1], s_addr[2], s_addr[3]);
        unsafe {
            ptr::copy(addr.as_ptr().cast::<c_char>(), dst, addr.len());
        }
        dst
    }
}

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/inet_ntop.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn inet_pton(af: c_int, src: *const c_char, dst: *mut c_void) -> c_int {
    if af != AF_INET {
        platform::ERRNO.set(EAFNOSUPPORT);
        -1
    } else {
        let s_addr = unsafe {
            slice::from_raw_parts_mut(
                ptr::from_mut(&mut (*dst.cast::<in_addr>()).s_addr).cast::<u8>(),
                4,
            )
        };
        let src_cstr = unsafe { CStr::from_ptr(src) };
        let mut octets = unsafe { str::from_utf8_unchecked(src_cstr.to_bytes()).split('.') };
        for part in s_addr.iter_mut().take(4) {
            if let Some(n) = octets
                .next()
                .filter(|x| !x.len() > 3)
                .and_then(|x| u8::from_str(x).ok())
            {
                *part = n;
            } else {
                return 0;
            }
        }
        if octets.next().is_none() {
            1 // Success
        } else {
            0
        }
    }
}
