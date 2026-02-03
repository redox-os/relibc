use crate::{
    header::stdio::{F_NORD, F_NOWR, FILE},
    platform::types::{c_int, size_t},
};

#[unsafe(no_mangle)]
pub extern "C" fn __fpending(stream: *mut FILE) -> size_t {
    let stream = unsafe { &mut *stream }.lock();

    stream.writer.pending()
}

#[unsafe(no_mangle)]
pub extern "C" fn __freadable(stream: *mut FILE) -> c_int {
    let stream = unsafe { &mut *stream }.lock();

    (stream.flags & F_NORD == 0) as c_int
}

#[unsafe(no_mangle)]
pub extern "C" fn __fwritable(stream: *mut FILE) -> c_int {
    let stream = unsafe { &mut *stream }.lock();

    (stream.flags & F_NOWR == 0) as c_int
}

//TODO: Check last operation when read-write
#[unsafe(no_mangle)]
pub extern "C" fn __freading(stream: *mut FILE) -> c_int {
    let stream = unsafe { &mut *stream }.lock();

    (stream.flags & F_NORD == 0) as c_int
}

//TODO: Check last operation when read-write
#[unsafe(no_mangle)]
pub extern "C" fn __fwriting(stream: *mut FILE) -> c_int {
    let stream = unsafe { &mut *stream }.lock();

    (stream.flags & F_NOWR == 0) as c_int
}
