//! `err.h` implementation.
//!
//! See <https://man.freebsd.org/cgi/man.cgi?err>
//!
//! `err.h` is a BSD extension to the C library which provides functions for printing formatted
//! errors. Errors are printed to [`stdio::stderr`] by default or to a file set by
//! [`err_set_file`]. This family of functions is non-portable, but it is also supported by `glibc`
//! and `musl`.
//!
//! The functions come in sets of three. Each of them print the program binary name (the last path
//! segment of arg[0]) and an optional user message along with these differences:
//! * No suffix: Prints an error message for ERRNO based on [`strerror`]
//! * `c` suffix: Prints an error message for an arbitrary error code
//! * `x` suffix: Does not print an error code
//!
//! For example, `err` does not have a suffix so it would print the program name, the user message,
//! and an error string for ERRNO. `errc` would operate in the same way except the functions takes
//! an error code for which to print an error string.

// Allow is intentional. Almost every line of the simple functions below are unsafe.
// unsafe_op_in_unsafe_fn only adds visual noise or a needless indentation here.
#![allow(unsafe_op_in_unsafe_fn)]

use core::{
    ffi::{c_char, c_int, VaList as va_list},
    ptr,
};

use crate::{
    c_str::CStr,
    header::{
        stdio::{self, fprintf, fputc, fputs, vfprintf, FILE},
        stdlib::exit,
        string::strerror,
    },
    platform::{self, ERRNO},
};

// Optional callback from user invoked on exit.
type ExitCallback = Option<unsafe extern "C" fn(c_int)>;
static mut on_exit: ExitCallback = None;

// Messages from this module are written to this sink.
static mut error_sink: *mut FILE = ptr::null_mut();

/// Set global [`FILE`] sink to write errors and warnings.
#[no_mangle]
pub unsafe extern "C" fn err_set_file(fp: *mut FILE) {
    if fp.is_null() {
        error_sink = stdio::stderr;
    } else {
        error_sink = fp;
    }
}

/// Set or remove a callback to invoke before exiting on error.
#[no_mangle]
pub unsafe extern "C" fn err_set_exit(ef: ExitCallback) {
    on_exit = ef;
}

/// Print a user message then an error message for [`ERRNO`] followed by exiting with `eval`.
///
/// The message format is `progname: fmt: strerror(ERRNO)`
///
/// # Return
/// Does not return. Exits with `eval` as an error code.
#[no_mangle]
pub unsafe extern "C" fn err(eval: c_int, fmt: *const c_char, mut va_list: ...) -> ! {
    let code = Some(ERRNO.get());
    err_exit(eval, code, fmt, va_list.as_va_list())
}

/// Print a user message then an error message for `code` before exiting with `eval` as a return.
///
/// The message format is `progname: fmt: strerror(code)`
///
/// # Return
/// Exits with `eval` as an error code.
#[no_mangle]
pub unsafe extern "C" fn errc(eval: c_int, code: c_int, fmt: *const c_char, mut va_list: ...) -> ! {
    err_exit(eval, Some(code), fmt, va_list.as_va_list())
}

/// Print a user message then exits with `eval` as a return.
///
/// The message format is `progname: fmt`
///
/// # Return
/// Exits with `eval` as an error code.
#[no_mangle]
pub unsafe extern "C" fn errx(eval: c_int, fmt: *const c_char, mut va_list: ...) -> ! {
    err_exit(eval, None, fmt, va_list.as_va_list())
}

/// Print a user message and then an error message for [`ERRNO`].
///
/// The message format is `progname: fmt: strerror(ERRNO)`
#[no_mangle]
pub unsafe extern "C" fn warn(fmt: *const c_char, mut va_list: ...) {
    let code = Some(ERRNO.get());
    display_message(code, fmt, va_list.as_va_list());
}

/// Print a user message then an error message for `code`.
///
/// The message format is `progname: fmt: strerror(code)`
#[no_mangle]
pub unsafe extern "C" fn warnc(code: c_int, fmt: *const c_char, mut va_list: ...) {
    display_message(Some(code), fmt, va_list.as_va_list());
}

/// Print a user message as a warning.
///
/// The message format is `progname: fmt`
#[no_mangle]
pub unsafe extern "C" fn warnx(fmt: *const c_char, mut va_list: ...) {
    display_message(None, fmt, va_list.as_va_list());
}

/// See [`err`].
#[no_mangle]
pub unsafe extern "C" fn verr(eval: c_int, fmt: *const c_char, args: va_list) -> ! {
    let code = Some(ERRNO.get());
    err_exit(eval, code, fmt, args);
}

/// See [`errc`].
#[no_mangle]
pub unsafe extern "C" fn verrc(eval: c_int, code: c_int, fmt: *const c_char, args: va_list) -> ! {
    err_exit(eval, Some(code), fmt, args)
}

/// See [`errx`];
#[no_mangle]
pub unsafe extern "C" fn verrx(eval: c_int, fmt: *const c_char, args: va_list) -> ! {
    err_exit(eval, None, fmt, args)
}

/// See [`warn`].
#[no_mangle]
pub unsafe extern "C" fn vwarn(fmt: *const c_char, args: va_list) {
    let code = Some(ERRNO.get());
    display_message(code, fmt, args);
}

/// See [`warnc`].
#[no_mangle]
pub unsafe extern "C" fn vwarnc(code: c_int, fmt: *const c_char, args: va_list) {
    display_message(Some(code), fmt, args);
}

/// See [`warnx`].
#[no_mangle]
pub unsafe extern "C" fn vwarnx(fmt: *const c_char, args: va_list) {
    display_message(None, fmt, args);
}

// Write error messages for err and warn to the currently set sink.
unsafe fn display_message(code: Option<c_int>, fmt: *const c_char, args: va_list) {
    /// SAFETY:
    /// * error_sink is only null once on start but otherwise always stderr or a user set file
    /// * User is trusted to pass in a valid file pointer if err_set_file is used
    if error_sink.is_null() {
        error_sink = stdio::stderr;
    }
    let sink = error_sink;

    // "progname:" is always printed
    // SAFETY:
    // * program_invocation_short_name is never null as it is set on start
    // * program_invocation_short_name is not globally mutable so the user can't mangle it
    fprintf(
        sink,
        c"%s".as_ptr(),
        platform::program_invocation_short_name,
    );

    // Print user message if any
    if !fmt.is_null() {
        fputs(c": ".as_ptr(), sink);
        vfprintf(sink, fmt, args);
    }

    // Print error message for non-x functions
    if let Some(code) = code {
        let message = strerror(code);
        fprintf(sink, c": %s".as_ptr(), message);
    }

    // Always write new line
    fputc(b'\n'.into(), sink);
}

// Write an error message as per err and then exit.
unsafe fn err_exit(eval: c_int, code: Option<c_int>, fmt: *const c_char, args: va_list) -> ! {
    display_message(code, fmt, args);

    if let Some(callback) = on_exit {
        // errx will hit the unwrap.
        callback(code.unwrap_or_else(|| ERRNO.get()));
    }

    exit(eval);
}
