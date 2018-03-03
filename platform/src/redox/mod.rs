pub fn exit(status: c_int) -> ! {
    syscall::exit(status);
}

pub fn write(fd: c_int, buf: &[u8]) -> ssize_t {
    syscall::write(fd, buf);
    buf.len() as ssize_t
}
