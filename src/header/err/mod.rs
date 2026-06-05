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
//! segment of `argv[0]`) and an optional user message along with these differences:
//! * No suffix: Prints an error message for ERRNO based on [`strerror`]
//! * `c` suffix: Prints an error message for an arbitrary error code
//! * `x` suffix: Does not print an error code
//!
//! For example, `err` does not have a suffix so it would print the program name, the user message,
//! and an error string for ERRNO. `errc` would operate in the same way except the functions takes
//! an error code for which to print an error string.

use core::{
    ffi::{VaList as va_list, c_char, c_int},
    ptr,
};

use crate::{
    header::{
        stdio::{self, FILE, fprintf, fputc, fputs, vfprintf},
        stdlib::exit,
        string::strerror,
    },
    platform::{self, ERRNO},
};

// Optional callback from user invoked on exit.
type ExitCallback = Option<unsafe extern "C" fn(c_int)>;
static mut ON_EXIT: ExitCallback = None;

// Messages from this module are written to this sink.
static mut ERROR_SINK: *mut FILE = ptr::null_mut();

/// Set global [`FILE`] sink to write errors and warnings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn err_set_file(fp: *mut FILE) {
    if fp.is_null() {
        unsafe {
            ERROR_SINK = stdio::stderr;
        }
    } else {
        unsafe {
            ERROR_SINK = fp;
        }
    }
}

/// Set or remove a callback to invoke before exiting on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn err_set_exit(ef: ExitCallback) {
    unsafe {
        ON_EXIT = ef;
    }
}

/// Print a user message then an error message for [`ERRNO`] followed by exiting with `eval`.
///
/// The message format is `progname: fmt: strerror(ERRNO)`
///
/// # Return
/// Does not return. Exits with `eval` as an error code.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn err(eval: c_int, fmt: *const c_char, va_list: ...) -> ! {
    let code = Some(ERRNO.get());
    unsafe { err_exit(eval, code, fmt, va_list) }
}

/// Print a user message then an error message for `code` before exiting with `eval` as a return.
///
/// The message format is `progname: fmt: strerror(code)`
///
/// # Return
/// Exits with `eval` as an error code.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn errc(eval: c_int, code: c_int, fmt: *const c_char, va_list: ...) -> ! {
    unsafe { err_exit(eval, Some(code), fmt, va_list) }
}

/// Print a user message then exits with `eval` as a return.
///
/// The message format is `progname: fmt`
///
/// # Return
/// Exits with `eval` as an error code.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn errx(eval: c_int, fmt: *const c_char, va_list: ...) -> ! {
    unsafe { err_exit(eval, None, fmt, va_list) }
}

/// Print a user message and then an error message for [`ERRNO`].
///
/// The message format is `progname: fmt: strerror(ERRNO)`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn warn(fmt: *const c_char, va_list: ...) {
    let code = Some(ERRNO.get());
    unsafe {
        display_message(code, fmt, va_list);
    }
}

/// Print a user message then an error message for `code`.
///
/// The message format is `progname: fmt: strerror(code)`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn warnc(code: c_int, fmt: *const c_char, va_list: ...) {
    unsafe {
        display_message(Some(code), fmt, va_list);
    }
}

/// Print a user message as a warning.
///
/// The message format is `progname: fmt`
#[unsafe(no_mangle)]
pub unsafe extern "C" fn warnx(fmt: *const c_char, va_list: ...) {
    unsafe {
        display_message(None, fmt, va_list);
    }
}

/// See [`err`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn verr(eval: c_int, fmt: *const c_char, args: va_list) -> ! {
    let code = Some(ERRNO.get());
    unsafe {
        err_exit(eval, code, fmt, args);
    }
}

/// See [`errc`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn verrc(eval: c_int, code: c_int, fmt: *const c_char, args: va_list) -> ! {
    unsafe { err_exit(eval, Some(code), fmt, args) }
}

/// See [`errx`];
#[unsafe(no_mangle)]
pub unsafe extern "C" fn verrx(eval: c_int, fmt: *const c_char, args: va_list) -> ! {
    unsafe { err_exit(eval, None, fmt, args) }
}

/// See [`warn`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vwarn(fmt: *const c_char, args: va_list) {
    let code = Some(ERRNO.get());
    unsafe {
        display_message(code, fmt, args);
    }
}

/// See [`warnc`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vwarnc(code: c_int, fmt: *const c_char, args: va_list) {
    unsafe {
        display_message(Some(code), fmt, args);
    }
}

/// See [`warnx`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vwarnx(fmt: *const c_char, args: va_list) {
    unsafe {
        display_message(None, fmt, args);
    }
}

// Write error messages for err and warn to the currently set sink.
unsafe fn display_message(code: Option<c_int>, fmt: *const c_char, args: va_list) {
    // SAFETY:
    // * ERROR_SINK is only null once on start but otherwise always stderr or a user set file
    // * User is trusted to pass in a valid file pointer if err_set_file is used
    if unsafe { ERROR_SINK.is_null() } {
        unsafe {
            ERROR_SINK = stdio::stderr;
        }
    }
    let sink = unsafe { ERROR_SINK };

    // "progname:" is always printed
    // SAFETY:
    // * program_invocation_short_name is never null as it is set on start
    // * program_invocation_short_name is not globally mutable so the user can't mangle it
    unsafe {
        fprintf(
            sink,
            c"%s".as_ptr(),
            platform::program_invocation_short_name,
        );
    }

    // Print user message if any
    if !fmt.is_null() {
        unsafe {
            fputs(c": ".as_ptr(), sink);
            vfprintf(sink, fmt, args);
        }
    }

    // Print error message for non-x functions
    if let Some(code) = code {
        unsafe {
            let message = strerror(code);
            fprintf(sink, c": %s".as_ptr(), message);
        }
    }

    // Always write new line
    unsafe {
        fputc(b'\n'.into(), sink);
    }
}

// Write an error message as per err and then exit.
unsafe fn err_exit(eval: c_int, code: Option<c_int>, fmt: *const c_char, args: va_list) -> ! {
    unsafe {
        display_message(code, fmt, args);
    }

    if let Some(callback) = unsafe { ON_EXIT } {
        // errx will hit the unwrap.
        unsafe {
            callback(code.unwrap_or_else(|| ERRNO.get()));
        }
    }

    unsafe {
        exit(eval);
    }
}
