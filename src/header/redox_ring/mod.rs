use crate::platform::types::{c_char, c_int, c_uint, c_void, size_t, uint8_t, uint64_t};

use core::{mem, ptr};

pub use crate::platform::sys::ring::{self, pool, redox_ring_cq, redox_ring_sq};

/// Opaque handle for user programs.
///
/// Fields must not be accessed directly from user programs.
/// Use the `redox_ring_*()` API functions to manipulate this structure.
pub struct redox_ring<'ring> {
    pub sq: redox_ring_sq<'ring>,
    pub cq: redox_ring_cq<'ring>,
    pub flags: c_uint,
    pub ring_fd: usize,
    pub pool: *mut pool<'ring>,
    pub features: c_uint,
    pub int_flags: uint8_t,
    pub offset: u64,
}

#[repr(C)]
#[derive(Debug)]
pub struct redox_ring_params {
    pub sq_entries: c_uint,
    pub cq_entries: c_uint,
    pub sqe_size: c_uint,
    pub cqe_size: c_uint,
    pub pool_size: c_uint,
    pub chunk_size: c_uint,
    pub sized: c_uint,
    pub flags: c_uint,
    pub sq_thread_cpu: c_uint,
    pub sq_thread_idle: c_uint,
    pub features: c_uint,
    pub wq_fd: c_uint,
    pub resv: [c_uint; 3],
    pub sq_off: io_sqring_offsets,
    pub cq_off: io_cqring_offsets,
}

#[repr(C)]
#[derive(Debug)]
pub struct io_sqring_offsets {
    pub head: c_uint,
    pub tail: c_uint,
    pub ring_mask: c_uint,
    pub ring_entries: c_uint,
    pub flags: c_uint,
    pub dropped: c_uint,
    pub array: c_uint,
    pub resv1: c_uint,
    pub user_addr: uint64_t,
}

#[repr(C)]
#[derive(Debug)]
pub struct io_cqring_offsets {
    pub head: c_uint,
    pub tail: c_uint,
    pub ring_mask: c_uint,
    pub ring_entries: c_uint,
    pub overflow: c_uint,
    pub cqes: c_uint,
    pub flags: c_uint,
    pub resv1: c_uint,
    pub user_addr: uint64_t,
}

/// # Return
///
/// Returns 0 on success, or `-errno` on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_ring_register_files(
    ring: *mut redox_ring,
    files: *const c_int,
    nr_files: c_uint,
) -> c_int {
    syscall::Error::mux(
        unsafe { ring::__redox_ring_register_files(ring, files, nr_files) }.map(|()| 0),
    ) as c_int
}

/// # Return
///
/// Returns 0 on success, or `-errno` on error.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_ring_queue_init_with_path(
    entries: c_uint,
    sqe_size: c_uint,
    cqe_size: c_uint,
    pool_size: c_uint,
    chunk_size: c_uint,
    sized: c_uint,
    ring: *mut redox_ring,
    flags: c_uint,
    path: *const c_char,
) -> c_int {
    let mut p: redox_ring_params = unsafe { mem::zeroed() };

    p.sq_entries = entries;
    p.cq_entries = entries;
    p.sqe_size = sqe_size;
    p.cqe_size = cqe_size;
    p.pool_size = pool_size;
    p.chunk_size = chunk_size;
    p.sized = sized;
    p.flags = flags;

    syscall::Error::mux(unsafe {
        ring::__redox_ring_queue_init_params_with_path(
            ring,
            &raw mut p,
            ptr::null_mut::<c_void>(),
            0,
            path,
        )
        .map(|()| 0)
    }) as c_int
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_ring_alloc_buf(
    ring: *mut redox_ring,
    out_ptr: *mut *mut u8,
) -> c_int {
    unsafe { ring::redox_ring_alloc_buf(ring, out_ptr) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_ring_alloc_sized(
    ring: *mut redox_ring,
    size: c_uint,
    align: c_uint,
    out_ptr: *mut *mut u8,
) -> c_int {
    unsafe { ring::redox_ring_alloc_buf_sized(ring, size, align, out_ptr) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_ring_free_buf(ring: *mut redox_ring, offset: c_uint) -> c_int {
    unsafe { ring::redox_ring_free_buf(ring, ptr::null_mut(), offset) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_ring_free_buf_sized(ring: *mut redox_ring, ptr: *mut u8) -> c_int {
    unsafe { ring::redox_ring_free_buf(ring, ptr, 0) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_ring_get_sqe(ring: *mut redox_ring) -> *mut c_void {
    unsafe { ring::redox_ring_get_sqe(ring) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_ring_submit(ring: *mut redox_ring) -> c_int {
    unsafe { ring::redox_ring_submit(ring) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_ring_wait_cqe(ring: *mut redox_ring, cqe: *mut c_void) -> c_int {
    unsafe { ring::redox_ring_wait_cqe(ring, cqe) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_ring_cqe_seen(ring: *mut redox_ring) {
    unsafe { ring::redox_ring_cqe_seen(ring) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn redox_ring_queue_exit(ring: *mut redox_ring) {
    unsafe { ring::redox_ring_queue_exit(ring) }
}

#[unsafe(no_mangle)]
pub extern "C" fn redox_ring_sizeof() -> size_t {
    core::mem::size_of::<redox_ring>() as size_t
}
