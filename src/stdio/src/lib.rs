//! stdio implementation for Redox, following http://pubs.opengroup.org/onlinepubs/7908799/xsh/stdio.h.html

#![no_std]

extern crate errno;
extern crate platform;
extern crate va_list as vl;

use core::str;
use core::fmt::Write;

use platform::types::*;
use platform::c_str;
use platform::errno;
use errno::STR_ERROR;
use vl::VaList as va_list;

mod printf;

pub const BUFSIZ: c_int = 4096;

pub const FILENAME_MAX: c_int = 4096;

pub type fpos_t = off_t;

pub struct FILE;

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut stdout: *mut FILE = 1 as *mut FILE;

#[allow(non_upper_case_globals)]
#[no_mangle]
pub static mut stderr: *mut FILE = 2 as *mut FILE;

#[no_mangle]
pub extern "C" fn clearerr(stream: *mut FILE) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ctermid(s: *mut c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn cuserid(s: *mut c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fclose(stream: *mut FILE) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fdopen(fildes: c_int, mode: *const c_char) -> *mut FILE {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn feof(stream: *mut FILE) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ferror(stream: *mut FILE) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fflush(stream: *mut FILE) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fgetc(stream: *mut FILE) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fgetpos(stream: *mut FILE, pos: *mut fpos_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fgets(s: *mut c_char, n: c_int, stream: *mut FILE) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fileno(stream: *mut FILE) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn flockfile(file: *mut FILE) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fopen(filename: *const c_char, mode: *const c_char) -> *mut FILE {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fputc(c: c_int, stream: *mut FILE) -> c_int {
    platform::FileWriter(stream as c_int).write_char(c as u8 as char).map_err(|_| return -1);
    c
}

#[no_mangle]
pub unsafe extern "C" fn fputs(s: *const c_char, stream: *mut FILE) -> c_int {
    extern "C" {
        fn strlen(s: *const c_char) -> size_t;
    }
    use core::{ str, slice };
    let len = strlen(s);
    platform::FileWriter(stream as c_int).write_str(str::from_utf8_unchecked(slice::from_raw_parts(
                        s as *const u8,
                        len,
                    ))).map_err(|_| return -1);
    len as i32
}

#[no_mangle]
pub extern "C" fn fread(ptr: *mut c_void, size: usize, nitems: usize, stream: *mut FILE) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn freopen(
    filename: *const c_char,
    mode: *const c_char,
    stream: *mut FILE,
) -> *mut FILE {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fseek(stream: *mut FILE, offset: c_long, whence: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fseeko(stream: *mut FILE, offset: off_t, whence: c_int) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fsetpos(stream: *mut FILE, pos: *const fpos_t) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ftell(stream: *mut FILE) -> c_long {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ftello(stream: *mut FILE) -> off_t {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ftrylockfile(file: *mut FILE) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn funlockfile(file: *mut FILE) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn fwrite(
    ptr: *const c_void,
    size: usize,
    nitems: usize,
    stream: *mut FILE,
) -> usize {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getc(stream: *mut FILE) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getchar() -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getc_unlocked(stream: *mut FILE) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getchar_unlocked() -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn gets(s: *mut c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn getw(stream: *mut FILE) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn pclose(stream: *mut FILE) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn perror(s: *const c_char) {
    let s_str = str::from_utf8_unchecked(c_str(s));

    let mut w = platform::FileWriter(2);
    if errno >= 0 && errno < STR_ERROR.len() as c_int {
        w.write_fmt(format_args!("{}: {}\n", s_str, STR_ERROR[errno as usize]));
    } else {
        w.write_fmt(format_args!("{}: Unknown error {}\n", s_str, errno));
    }
}

#[no_mangle]
pub extern "C" fn popen(command: *const c_char, mode: *const c_char) -> *mut FILE {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn putc(c: c_int, stream: *mut FILE) -> c_int {
    fputc(c, stream)
}

#[no_mangle]
pub unsafe extern "C" fn putchar(c: c_int) -> c_int {
    putc(c, stdout)
}

#[no_mangle]
pub extern "C" fn putc_unlocked(c: c_int, stream: *mut FILE) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn putchar_unlocked(c: c_int) -> c_int {
    putc_unlocked(c, stdout)
}

#[no_mangle]
pub unsafe extern "C" fn puts(s: *const c_char) -> c_int {
    fputs(s, stdout);
    putchar(b'\n' as c_int)
}

#[no_mangle]
pub extern "C" fn putw(w: c_int, stream: *mut FILE) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn remove(path: *const c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn rename(old: *const c_char, new: *const c_char) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn rewind(stream: *mut FILE) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setbuf(stream: *mut FILE, buf: *mut c_char) {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn setvbuf(stream: *mut FILE, buf: *mut c_char, mode: c_int, size: usize) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn tempnam(dir: *const c_char, pfx: *const c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn tmpfile() -> *mut FILE {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn tmpnam(s: *mut c_char) -> *mut c_char {
    unimplemented!();
}

#[no_mangle]
pub extern "C" fn ungetc(c: c_int, stream: *mut FILE) -> c_int {
    unimplemented!();
}

#[no_mangle]
pub unsafe extern "C" fn vfprintf(file: *mut FILE, format: *const c_char, ap: va_list) -> c_int {
    printf::printf(platform::FileWriter(file as c_int), format, ap)
}

#[no_mangle]
pub unsafe extern "C" fn vprintf(format: *const c_char, ap: va_list) -> c_int {
    vfprintf(stdout, format, ap)
}

#[no_mangle]
pub unsafe extern "C" fn vsnprintf(
    s: *mut c_char,
    n: usize,
    format: *const c_char,
    ap: va_list,
) -> c_int {
    printf::printf(platform::StringWriter(s as *mut u8, n as usize), format, ap)
}

#[no_mangle]
pub unsafe extern "C" fn vsprintf(s: *mut c_char, format: *const c_char, ap: va_list) -> c_int {
    printf::printf(platform::UnsafeStringWriter(s as *mut u8), format, ap)
}

/*
#[no_mangle]
pub extern "C" fn func(args) -> c_int {
    unimplemented!();
}
*/
