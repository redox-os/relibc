use syscall;
use libc::{c_uint, c_int, c_char, gid_t, uid_t, c_void, c_long, mode_t};
use syscall::error::{Error, EACCES, EPERM, EINVAL};

#[allow(non_camel_case_types)]
type clock_t = c_long;

macro_rules! UNIMPL {
    // Call with arguments and return value
    ($func:ident, $err:ident) => {{
         let err = Error::new($err);
         let _ = syscall::write(2, format!("unimplemented: {}: {}\n",
             stringify!($func), err).as_bytes());
         Err(err)
    }};
}

libc_fn!(alarm(_seconds: c_uint) -> c_uint {
    let _ = syscall::write(2, "unimplemented: alarm\n".as_bytes());
    0
});

libc_fn!(chown(_path: *mut c_char, _order: uid_t, _group: gid_t) -> Result<c_int> {
    UNIMPL!(chown, EACCES)
});

libc_fn!(_getdtablesize() -> Result<c_int> {
    Ok(65536)
});

// XXX variadic
libc_fn!(_ioctl(_file: c_int, _request: c_int) -> Result<c_int> {
    UNIMPL!(_ioctl, EINVAL)
});

libc_fn!(_link(_old: *const c_char, _new: *const c_char) -> Result<c_int> {
    UNIMPL!(_link, EPERM)
});

/*
libc_fn!(sysconf(_name: c_int) -> Result<c_long> {
    UNIMPL!(sysconf, EINVAL)
});
*/

// XXX type of argument pointer
libc_fn!(_times(_buf: *mut c_void) -> Result<clock_t> {
    UNIMPL!(_times, EINVAL)
});

libc_fn!(umask(_mode: mode_t) -> mode_t {
    // All permissions granted
    0o000
});

libc_fn!(ttyname(_fd: c_int) -> Result<*const c_char> {
    UNIMPL!(ttyname, EINVAL)
});

libc_fn!(fpathconf(_fildes: c_int, _name: c_int) -> Result<c_long> {
    UNIMPL!(fpathconf, EINVAL)
});

libc_fn!(getlogin() -> Result<*const c_char> {
    UNIMPL!(getlogin, EINVAL)
});
