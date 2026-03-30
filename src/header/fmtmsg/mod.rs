//! `fmtmsg.h` implementation.
//! 
//! See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/fmtmsg.h.html>.
//! Ported from Musl.
//! 
//! See <https://github.com/kraj/musl/blob/kraj/master/src/misc/fmtmsg.c>

use crate::{
    header::{
        fcntl::open,
        pthread::{PTHREAD_CANCEL_DISABLE, pthread_setcancelstate},
        stdio::dprintf,
        stdlib::getenv,
        string::strchr,
        unistd::close,
    },
    platform::types::{c_char, c_int, c_long},
};
pub const MM_NULLSEV: c_int = 0;
pub const MM_HALT: c_int = 1;
pub const MM_ERROR: c_int = 2;
pub const MM_WARNING: c_int = 3;
pub const MM_INFO: c_int = 4;
pub const MM_NOMSG: c_int = 1;
pub const MM_NOTOK: c_int = -1;
// c_long because classification is a c_long
pub const MM_PRINT: c_long = 256;
// c_long because classification is a c_long
pub const MM_CONSOLE: c_long = 512;
pub const MM_NOCON: c_int = 4;
pub const O_WRONLY: c_int = 0x01;

/*
 * If lstr is the first part of bstr, check that the next char in bstr
 * is either \0 or ':'
 */
unsafe fn strcolcmp(mut lstr: *const c_char, mut bstr: *const c_char) -> c_int {
    unsafe {
        // We already know lstr and bstr are non-null
        while *lstr != 0 && *bstr != 0 && (*lstr == *bstr) {
            lstr = lstr.add(1);
            bstr = bstr.add(1);
        }
        if *lstr != 0 || (*bstr != 0 && *bstr != b':' as c_char) {
            1
        } else {
            0
        }
    }
}
/// See <https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/fmtmsg.h.html>
#[unsafe(no_mangle)]
pub unsafe extern "C" fn fmtmsg(
    classification: c_long,
    label: *const c_char,
    severity: c_int,
    text: *const c_char,
    action: *const c_char,
    tag: *const c_char,
) -> c_int {
    let mut ret: c_int = 0;
    let mut verb: c_int = 0;
    let mut i: usize;
    let consolefd: c_int;
    let mut cs: c_int = 0;
    unsafe {
        pthread_setcancelstate(PTHREAD_CANCEL_DISABLE, &mut cs as *mut c_int);
    }
    let mut cmsg = unsafe { getenv(c"MSGVERB".as_ptr()) };
    let errstring: *const c_char = match severity {
        MM_HALT => c"HALT: ".as_ptr(),
        MM_ERROR => c"ERROR: ".as_ptr(),
        MM_WARNING => c"WARNING: ".as_ptr(),
        MM_INFO => c"INFO: ".as_ptr(),
        _ => MM_NULLSEV as _, // Default
    };
    let msgs: [*const c_char; 6] = [
        c"label".as_ptr(),
        c"severity".as_ptr(),
        c"text".as_ptr(),
        c"action".as_ptr(),
        c"tag".as_ptr(),
        core::ptr::null(),
    ];
    if (classification & MM_CONSOLE) != 0 {
        //TODO: rusty
        unsafe {
            consolefd = open(c"/dev/console".as_ptr(), O_WRONLY);
            if consolefd < 0 {
                ret = MM_NOCON;
            } else {
                #[rustfmt::skip]
                let status = dprintf(
                    consolefd,
                    c"%s%s%s%s%s%s%s%s\n".as_ptr(),
                    if !label.is_null() {label } else { c"".as_ptr()},
                    if !label.is_null() {c": ".as_ptr()} else {  c"".as_ptr() }, 
                    if severity != 0 { errstring } else { c"".as_ptr()},
                    if !text.is_null() { text } else { c"".as_ptr() },
                    if !action.is_null() {c"\nTO FIX: ".as_ptr()} else { c"".as_ptr()},
                    if !action.is_null() {action} else {c"".as_ptr()},
                    if !action.is_null() {c" ".as_ptr()} else { c"".as_ptr()},
                    if !tag.is_null() { tag } else { c"".as_ptr() },
                );
                if status < 1 {
                    ret = MM_NOCON;
                }
                close(consolefd);
            }
        }
    }
    if (classification & MM_PRINT) != 0 {
        unsafe {
            while !cmsg.is_null() && *cmsg != 0 {
                i = 0;
                loop {
                    if msgs[i].is_null() {
                        break;
                    }
                    if strcolcmp(msgs[i], cmsg) == 0 {
                        break;
                    }
                    i += 1;
                }
                if msgs[i].is_null() {
                    //ignore MSGVERB-unrecognized component
                    verb = 0xFF;
                    break;
                } else {
                    verb |= 1 << i;
                    cmsg = strchr(cmsg, b':' as _);
                    if !cmsg.is_null() {
                        cmsg = cmsg.add(1);
                    }
                }
            }
            if verb == 0 {
                verb = 0xFF;
            }
            #[rustfmt::skip]
            let status = dprintf(2,c"%s%s%s%s%s%s%s%s\n".as_ptr(),
                if (verb & 1 != 0) && !label.is_null() { label } else { c"".as_ptr() },
                    if (verb & 1 != 0) && !label.is_null() { c": ".as_ptr() } else { c"".as_ptr() },
                    if (verb & 2 != 0) && severity != 0 { errstring } else { c"".as_ptr() },
                    if (verb & 4 != 0) && !text.is_null() { text } else { c"".as_ptr() },
                    if (verb & 8 != 0) && !action.is_null() { c"\nTO FIX: ".as_ptr() } else { c"".as_ptr() },
                    if (verb & 8 != 0) && !action.is_null() { action } else { c"".as_ptr() },
                    if (verb & 8 != 0) && !action.is_null() { c" ".as_ptr() } else { c"".as_ptr() },
                    if (verb & 16 != 0) && !tag.is_null() { tag } else { c"".as_ptr() },
            );
            if status < 1 {
                ret |= MM_NOMSG;
            }
        }
    }
    if ret & (MM_NOCON | MM_NOMSG) == (MM_NOCON | MM_NOMSG) {
        ret = MM_NOTOK;
    }
    ret
}