use core::slice;

pub unsafe fn cstr_to_slice<'a>(buf: *const c_char) -> &'a [u8] {
    slice::from_raw_parts(buf as *const u8, ::strlen(buf) as usize)
}

pub fn chdir(path: *const c_char) -> c_int {
    syscall::chdir(cstr_to_slice(path))? as c_int
}

pub fn close(fildes: c_int) -> c_int {
    syscall::close(fildes as usize)? as c_int
}

pub fn dup(fildes: c_int) -> c_int {
    syscall::dup(file as usize, &[])? as c_int
}

pub fn dup2(fildes: c_int, fildes2) -> c_int {
    syscall::dup2(fildes as usize, fildes2 as usize, &[])? as c_int

pub fn exit(status: c_int) -> ! {
    syscall::exit(status);
}

pub fn write(fd: c_int, buf: &[u8]) -> ssize_t {
    syscall::write(fd, buf);
    buf.len() as ssize_t
}
