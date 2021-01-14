use crate::{
    header::stdio::{FILE, F_NORD, F_NOWR},
    platform::types::*,
};

#[no_mangle]
pub extern "C" fn __fpending(stream: *mut FILE) -> size_t {
    let stream = unsafe { &mut *stream }.lock();

    stream.writer.pending()
}

#[no_mangle]
pub extern "C" fn __freadable(stream: *mut FILE) -> c_int {
    let stream = unsafe { &mut *stream }.lock();

    (stream.flags & F_NORD == 0) as c_int
}

#[no_mangle]
pub extern "C" fn __fwritable(stream: *mut FILE) -> c_int {
    let stream = unsafe { &mut *stream }.lock();

    (stream.flags & F_NOWR == 0) as c_int
}

//TODO: Check last operation when read-write
#[no_mangle]
pub extern "C" fn __freading(stream: *mut FILE) -> c_int {
    let stream = unsafe { &mut *stream }.lock();

    (stream.flags & F_NORD == 0) as c_int
}

//TODO: Check last operation when read-write
#[no_mangle]
pub extern "C" fn __fwriting(stream: *mut FILE) -> c_int {
    let stream = unsafe { &mut *stream }.lock();

    (stream.flags & F_NOWR == 0) as c_int
}
