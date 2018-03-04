use syscall;
use libc::{c_int, c_char, c_void, size_t};
use core::slice;

libc_fn!(unsafe redox_fevent(file: c_int, flags: c_int) -> Result<c_int> {
    Ok(syscall::fevent(file as usize, flags as usize)? as c_int)
});

libc_fn!(unsafe redox_fpath(file: c_int, buf: *mut c_char, len: size_t) -> Result<c_int> {
    let buf = slice::from_raw_parts_mut(buf as *mut u8, len);
    Ok(syscall::fpath(file as usize, buf)? as c_int)
});

libc_fn!(unsafe redox_fmap(file: c_int, offset: size_t, size: size_t) -> Result<*mut c_void> {
    Ok(syscall::fmap(file as usize, offset, size)? as *mut c_void)
});

libc_fn!(unsafe redox_funmap(addr: *mut c_void) -> Result<c_int> {
    Ok(syscall::funmap(addr as usize)? as c_int)
});

libc_fn!(unsafe redox_physalloc(size: size_t) -> Result<*mut c_void> {
    Ok(syscall::physalloc(size)? as *mut c_void)
});

libc_fn!(unsafe redox_physfree(physical_address: *mut c_void, size: size_t) -> Result<c_int> {
    Ok(syscall::physfree(physical_address as usize, size)? as c_int)
});

libc_fn!(unsafe redox_physmap(physical_address: *mut c_void, size: size_t, flags: c_int) -> Result<*mut c_void> {
    Ok(syscall::physmap(physical_address as usize, size, flags as usize)? as *mut c_void)
});

libc_fn!(unsafe redox_physunmap(virtual_address: *mut c_void) -> Result<c_int> {
    Ok(syscall::physunmap(virtual_address as usize)? as c_int)
});

libc_fn!(unsafe redox_virttophys(virtual_address: *mut c_void) -> Result<*mut c_void> {
    Ok(syscall::virttophys(virtual_address as usize)? as *mut c_void)
});
