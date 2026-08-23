use crate::{
    header::{
        bits_timespec::timespec,
        redox_ring::{redox_ring, redox_ring_params},
    },
    platform::types::{c_char, c_int, c_uint, c_void, size_t},
    sync::{futex_wait, futex_wake},
};

use core::{
    mem,
    num::NonZeroU32,
    ops::Deref,
    ptr::{self, NonNull},
    sync::atomic::Ordering,
};

use alloc::{boxed::Box, vec::Vec};
use redox_buffer_pool::{AllocationStrategy, BufferPool, NoHandle, marker};
use redox_rings::{
    ffi::{FfiRawConsumer, FfiRawProducer, FfiRingHeader, FfiRingPopError},
    raw::{FixedParameters, INDEX_MASK, WAITING_BIT},
    sync::{FutexWaitResult, SPIN_COUNT, WaitNotify},
};
use redox_rt::proc::FdGuard;
use syscall::{CallFlags, Error, TimeSpec};

impl From<crate::sync::FutexWaitResult> for FutexWaitResult {
    fn from(value: crate::sync::FutexWaitResult) -> Self {
        match value {
            crate::sync::FutexWaitResult::Waited => FutexWaitResult::Waited,
            crate::sync::FutexWaitResult::Stale => FutexWaitResult::Stale,
            crate::sync::FutexWaitResult::TimedOut => FutexWaitResult::TimedOut,
        }
    }
}

/*
 * Library interface to redox_ring
 */
pub const DEFAULT_SHM_SIZE: usize = 65_536;
pub const DEFAULT_CHUNK_SIZE: usize = 4096;
pub const HEADER_MMAP_OFFSET: usize = 256;

#[derive(Default)]
pub struct ShmAllocator {
    pub free_offsets: Vec<usize>,
    pub chunk_size: usize,
    pub total_size: usize,
}

impl ShmAllocator {
    pub fn new(total_size: usize, chunk_size: usize) -> Self {
        let num_chunks = total_size / chunk_size;
        let mut offsets = Vec::with_capacity(num_chunks);
        for i in (0..num_chunks).rev() {
            offsets.push(i * chunk_size);
        }
        Self {
            free_offsets: offsets,
            chunk_size,
            total_size,
        }
    }

    pub fn allocate(&mut self) -> Option<usize> {
        self.free_offsets.pop()
    }

    pub fn deallocate(&mut self, offset: usize) {
        self.free_offsets.push(offset);
    }
}

pub struct PoolDynamic<'ring> {
    pub inner: BufferPool<'ring, usize, NoHandle, ()>,
    pub shm_base: *mut u8,
    pub total_size: usize,
}

pub enum RingPool<'ring> {
    Chunk {
        allocator: ShmAllocator,
        shm_base: *mut u8,
    },
    Dynamic(PoolDynamic<'ring>),
}

impl<'ring> RingPool<'ring> {
    pub fn base_ptr(&self) -> *mut u8 {
        match self {
            RingPool::Chunk { shm_base, .. } => *shm_base,
            RingPool::Dynamic(d) => d.shm_base,
        }
    }

    pub fn total_size(&self) -> usize {
        match self {
            RingPool::Chunk { allocator, .. } => allocator.total_size,
            RingPool::Dynamic(d) => d.total_size,
        }
    }

    pub fn allocate(&mut self, size: usize, align: usize) -> *mut u8 {
        match self {
            RingPool::Chunk {
                allocator,
                shm_base,
            } => match allocator.allocate() {
                Some(offset) => unsafe { shm_base.add(offset) },
                None => ptr::null_mut(),
            },
            RingPool::Dynamic(d) => {
                let Some(slice) = d.inner.acquire_borrowed_slice::<marker::NoGuard>(
                    size,
                    align,
                    AllocationStrategy::Optimal,
                ) else {
                    return ptr::null_mut();
                };

                let ptr = slice.as_ptr().cast::<u8>().cast_mut();
                // Delegate space release to the caller
                core::mem::forget(slice);
                ptr
            }
        }
    }

    pub unsafe fn deallocate(&mut self, ptr: *mut u8, offset: usize) {
        match self {
            RingPool::Chunk { allocator, .. } => {
                allocator.deallocate(offset);
            }
            RingPool::Dynamic(d) => {
                let target_offset = if !ptr.is_null() {
                    let ptr_offset = unsafe { ptr.offset_from(d.shm_base) };
                    if ptr_offset < 0 {
                        return;
                    }
                    ptr_offset as usize
                } else {
                    offset
                };

                unsafe {
                    d.inner.deallocate_at::<marker::NoGuard>(target_offset);
                }
            }
        }
    }
}

pub struct pool<'ring> {
    pub inner: RingPool<'ring>,
}

pub struct redox_ring_sq<'ring> {
    pub sq: FfiRawProducer<'ring>,
    pub kflags: *mut c_uint,
    pub kdropped: *mut c_uint,
    pub ring_sz: size_t,
    pub ring_ptr: *mut c_void,
    pub pendings: c_int,
    pub pipe: FdGuard,
}

pub struct redox_ring_cq<'ring> {
    pub cq: FfiRawConsumer<'ring>,
    pub kflags: *mut c_uint,
    pub koverflow: *mut c_uint,
    pub cqes: *mut c_void,
    pub cqe_sz: c_uint,

    pub ring_sz: size_t,
    pub ring_ptr: *mut c_void,
}

impl<'ring> redox_ring_sq<'ring> {
    pub fn from_fd(fd: FdGuard, item_size: u32, pipe: FdGuard) -> syscall::Result<Self> {
        if item_size == 0 {
            return Err(syscall::Error::new(syscall::EINVAL));
        }

        let shm_size = {
            let mut stat = syscall::data::Stat::default();
            syscall::fstat(fd.as_c_fd().unwrap() as usize, &mut stat)?;
            stat.st_size as usize
        };

        let map = syscall::data::Map {
            offset: 0,
            size: shm_size,
            flags: syscall::MapFlags::MAP_SHARED
                | syscall::MapFlags::PROT_WRITE
                | syscall::MapFlags::PROT_READ,
            address: 0,
        };

        let ptr_raw = unsafe { syscall::fmap(fd.as_c_fd().unwrap() as usize, &map)? as *mut u8 };
        let _ptr = NonNull::new(ptr_raw).ok_or(syscall::Error::new(syscall::EINVAL))?;

        let header_ptr = ptr_raw.cast::<FfiRingHeader>();
        let header_ref = unsafe { &*header_ptr };
        let header_ref_static: &'static FfiRingHeader = unsafe { core::mem::transmute(header_ref) };

        let available_bytes = shm_size - HEADER_MMAP_OFFSET;
        let raw_capacity = available_bytes / item_size as usize;

        let queue_len = if raw_capacity.is_power_of_two() {
            raw_capacity as u32
        } else {
            (raw_capacity.next_power_of_two() >> 1) as u32
        };

        if queue_len == 0 {
            return Err(syscall::Error::new(syscall::EINVAL));
        }

        let parameters = FixedParameters {
            ptr_queue: unsafe { ptr_raw.add(HEADER_MMAP_OFFSET) },
            queue_len,
            item_len: Some(NonZeroU32::new(item_size).expect("Item size is zero")),
        };

        let raw_producer = FfiRawProducer::new(parameters, header_ref_static)?;
        Ok(Self {
            sq: raw_producer,
            kflags: ptr::null_mut(),
            kdropped: ptr::null_mut(),
            ring_sz: shm_size,
            ring_ptr: ptr_raw.cast::<c_void>(),
            pendings: 0,
            pipe,
        })
    }

    pub unsafe fn get_sqe(&mut self) -> syscall::Result<*mut [u8]> {
        let index = self.pendings as u32;
        let Ok([a1, _a2]) = self.sq.push_areas(index) else {
            return Err(syscall::Error::new(syscall::EWOULDBLOCK));
        };

        if a1.1 == 0 {
            return Err(syscall::Error::new(syscall::EWOULDBLOCK));
        }

        let slot_ptr = a1.0;
        self.pendings += 1;

        Ok(core::ptr::slice_from_raw_parts_mut(
            slot_ptr,
            self.sq.item_size(),
        ))
    }

    pub fn submit(&mut self) -> c_int {
        let count = self.pendings;
        if count > 0 {
            self.sq.advance_push_area(count as usize);
            self.pendings = 0;
            self.notify_on_tail();
        }
        count
    }
}

impl<'ring> WaitNotify for redox_ring_sq<'ring> {
    fn wait_on_head(&self, expected_head: u32, timeout_opt: Option<&TimeSpec>) -> FutexWaitResult {
        futex_wait(
            self.sq.header.head.deref(),
            expected_head,
            timeout_opt.map(timespec::from).as_ref(),
        )
        .into()
    }

    fn notify_on_head(&self) {
        futex_wake(self.sq.header.head.deref(), 1);
    }

    fn wait_on_tail(&self, expected_tail: u32, timeout_opt: Option<&TimeSpec>) -> FutexWaitResult {
        futex_wait(
            self.sq.header.tail.deref(),
            expected_tail,
            timeout_opt.map(timespec::from).as_ref(),
        )
        .into()
    }

    fn notify_on_tail(&self) {
        let _ = self.pipe.write(&[0]);
    }
}

impl<'ring> redox_ring_cq<'ring> {
    pub fn from_fd(fd: FdGuard, item_size: u32) -> syscall::Result<Self> {
        if item_size == 0 {
            return Err(syscall::Error::new(syscall::EINVAL));
        }

        let shm_size = {
            let mut stat = syscall::data::Stat::default();
            syscall::fstat(fd.as_c_fd().unwrap() as usize, &mut stat)?;
            stat.st_size as usize
        };

        let map = syscall::data::Map {
            offset: 0,
            size: shm_size,
            flags: syscall::MapFlags::MAP_SHARED
                | syscall::MapFlags::PROT_WRITE
                | syscall::MapFlags::PROT_READ,
            address: 0,
        };

        let ptr_raw = unsafe { syscall::fmap(fd.as_c_fd().unwrap() as usize, &map)? as *mut u8 };
        let _ptr = NonNull::new(ptr_raw).ok_or(syscall::Error::new(syscall::EINVAL))?;

        let header_ptr = ptr_raw.cast::<FfiRingHeader>();
        let header_ref = unsafe { &*header_ptr };
        let header_ref_static: &'static FfiRingHeader = unsafe { core::mem::transmute(header_ref) };

        let available_bytes = shm_size - HEADER_MMAP_OFFSET;
        let raw_capacity = available_bytes / item_size as usize;

        let queue_len = if raw_capacity.is_power_of_two() {
            raw_capacity as u32
        } else {
            (raw_capacity.next_power_of_two() >> 1) as u32
        };

        if queue_len == 0 {
            return Err(syscall::Error::new(syscall::EINVAL));
        }

        let parameters = FixedParameters {
            ptr_queue: unsafe { ptr_raw.add(HEADER_MMAP_OFFSET) },
            queue_len,
            item_len: Some(NonZeroU32::new(item_size).expect("Item size is zero")),
        };

        let raw_consumer = FfiRawConsumer::new(parameters, header_ref_static)?;
        Ok(Self {
            cq: raw_consumer,
            kflags: ptr::null_mut(),
            koverflow: ptr::null_mut(),
            cqes: ptr::null_mut(),
            cqe_sz: item_size,
            ring_sz: shm_size,
            ring_ptr: ptr_raw.cast::<c_void>(),
        })
    }

    fn peek_sync(
        &mut self,
        deadline_opt: Option<&TimeSpec>,
    ) -> Result<*const [u8], FfiRingPopError> {
        'outer: loop {
            let current_tail_raw = self.cq.header.tail.load(Ordering::Relaxed);
            let current_tail = current_tail_raw & INDEX_MASK;

            match self.peek() {
                Ok(item) => {
                    let current_tail_raw = self.cq.header.tail.load(Ordering::Relaxed);
                    if current_tail_raw & WAITING_BIT != 0 {
                        self.notify_on_head();
                    }
                    return Ok(item);
                }
                Err(FfiRingPopError::Broken) => {
                    return Err(FfiRingPopError::Broken);
                }
                Err(FfiRingPopError::ItemSizeMismatch) => {
                    return Err(FfiRingPopError::ItemSizeMismatch);
                }
                Err(FfiRingPopError::Empty) => {
                    let current_head_logical = self.cq.cached_index & INDEX_MASK;
                    let head_with_flag = current_head_logical | WAITING_BIT;

                    // spin SPIN_COUNT times.
                    for _ in 0..SPIN_COUNT {
                        let fresh_tail = self.cq.header.tail.load(Ordering::Acquire) & INDEX_MASK;
                        if fresh_tail != current_tail {
                            self.cq
                                .header
                                .head
                                .store(current_head_logical, Ordering::Relaxed);
                            continue 'outer;
                        }
                        core::hint::spin_loop();
                    }
                    self.cq.header.head.store(head_with_flag, Ordering::Release);

                    let fresh_tail = self.cq.header.tail.load(Ordering::Acquire) & INDEX_MASK;
                    if fresh_tail != current_tail {
                        self.cq
                            .header
                            .head
                            .store(current_head_logical, Ordering::Relaxed);
                        continue;
                    }

                    match self.wait_on_tail(current_tail, deadline_opt) {
                        FutexWaitResult::TimedOut => {
                            return Err(FfiRingPopError::Empty);
                        }
                        FutexWaitResult::Waited | FutexWaitResult::Stale => {
                            // Woke up or value changed (Stale).
                            // Loop again to retry pop.
                            continue;
                        }
                    }
                }
            }
        }
    }

    pub fn peek(&mut self) -> Result<*const [u8], FfiRingPopError> {
        let [a1, _a2] = unsafe { self.cq.pop_areas(0) }?;
        Ok(core::ptr::slice_from_raw_parts(a1.0, self.cq.item_size()))
    }

    pub fn seen(&mut self) {
        self.cq.advance_pop_area(1);
    }
}

impl<'ring> WaitNotify for redox_ring_cq<'ring> {
    fn wait_on_head(&self, expected_head: u32, timeout_opt: Option<&TimeSpec>) -> FutexWaitResult {
        futex_wait(
            self.cq.header.head.deref(),
            expected_head,
            timeout_opt.map(timespec::from).as_ref(),
        )
        .into()
    }

    fn notify_on_head(&self) {
        futex_wake(self.cq.header.head.deref(), 1);
    }

    fn wait_on_tail(&self, expected_tail: u32, timeout_opt: Option<&TimeSpec>) -> FutexWaitResult {
        futex_wait(
            self.cq.header.tail.deref(),
            expected_tail,
            timeout_opt.map(timespec::from).as_ref(),
        )
        .into()
    }

    fn notify_on_tail(&self) {
        futex_wake(self.cq.header.tail.deref(), 1);
    }
}

pub unsafe fn __redox_ring_register_files(
    ring: *mut redox_ring,
    files: *const c_int,
    nr_files: c_uint,
) -> syscall::Result<()> {
    use redox_rings::op::RingCallVerb;

    let Some(ring) = (unsafe { ring.as_mut() }) else {
        return Err(Error::new(syscall::EINVAL));
    };

    if files.is_null() || nr_files == 0 {
        return Err(Error::new(syscall::EINVAL));
    }

    let fixed_ftbl = unsafe { core::slice::from_raw_parts(files, nr_files as usize) };

    let mut fds = Vec::with_capacity(1 + nr_files as usize);
    fds.push(ring.ring_fd);
    fds.extend(fixed_ftbl.iter().map(|&fd| fd as usize));

    redox_rt::sys::sys_call_ro(
        fds.as_slice(),
        &mut [],
        CallFlags::empty(),
        &[RingCallVerb::SetFileTable as u64],
    )?;

    Ok(())
}

pub unsafe fn __redox_ring_queue_init_params_with_path(
    ring: *mut redox_ring,
    p: *mut redox_ring_params,
    _buf: *mut c_void,
    _buf_size: size_t,
    path: *const c_char,
) -> syscall::Result<()> {
    use redox_rings::op::{RingCallVerb, RingSetupFlags, RingSetupParams};
    use zerocopy::IntoBytes;

    // `redox_ring_params` contains client-side configuration fields (e.g., `sized`, `chunk_size`).
    // Only the fields relevant to the scheme side are forwarded via `RingSetupParams`.
    let Some(params) = (unsafe { p.as_mut() }) else {
        return Err(syscall::Error::new(syscall::EINVAL));
    };

    let sqe_size = params.sqe_size;
    let cqe_size = params.cqe_size;
    let sized = params.sized;
    let chunk_size = params.chunk_size;

    let path_str = unsafe {
        core::ffi::CStr::from_ptr(path.cast::<_>())
            .to_str()
            .map_err(|_| syscall::Error::new(syscall::EINVAL))?
    };

    let dir_fd = FdGuard::open_into_upper(path_str, syscall::O_DIRECTORY)?;
    let ring_fd = dir_fd.dup_into_upper(b"uring")?;
    let flags = RingSetupFlags::from_bits(params.flags).ok_or(Error::new(syscall::EINVAL))?;
    let mut setup_params = RingSetupParams {
        nr_sq_entries: params.sq_entries,
        nr_cq_entries: params.cq_entries,
        flags: flags.bits(),
        pool_size: params.pool_size,
    };

    ring_fd.call_rw(
        setup_params.as_mut_bytes(),
        CallFlags::empty(),
        &[RingCallVerb::Setup as u64],
    )?;

    params.sq_entries = setup_params.nr_sq_entries;
    params.cq_entries = setup_params.nr_cq_entries;

    // FIXME use these two variables?
    let _sq_size = params.sq_entries as usize * sqe_size as usize + mem::size_of::<FfiRingHeader>();
    let _cq_size = params.cq_entries as usize * cqe_size as usize + mem::size_of::<FfiRingHeader>();

    let mut fd_buf = [usize::MAX; 3]; // [sq_shm_fd, cq_shm_fd, pipe_fd]
    let fd_bytes = unsafe {
        core::slice::from_raw_parts_mut(
            fd_buf.as_mut_ptr().cast::<u8>(),
            fd_buf.len() * core::mem::size_of::<usize>(),
        )
    };
    ring_fd.call_ro(fd_bytes, syscall::CallFlags::FD, &[])?;

    let (sq_shm_fd, cq_shm_fd, pipe) = (
        FdGuard::new(fd_buf[0]),
        FdGuard::new(fd_buf[1]),
        FdGuard::new(fd_buf[2]).openat("write", 0, 0)?,
    );

    let shm_ptr = unsafe {
        syscall::fmap(
            ring_fd.as_raw_fd(),
            &syscall::data::Map {
                address: 0,
                size: params.pool_size as usize,
                flags: syscall::MapFlags::MAP_SHARED
                    | syscall::MapFlags::PROT_READ
                    | syscall::MapFlags::PROT_WRITE,
                offset: 0,
            },
        )? as *mut u8
    };

    let sq = redox_ring_sq::from_fd(sq_shm_fd, sqe_size, pipe)?;
    let cq = redox_ring_cq::from_fd(cq_shm_fd, cqe_size)?;

    let ring_pool_inner = if sized != 0 {
        let inner = BufferPool::new(None);
        match inner.begin_expand(params.pool_size as usize) {
            Ok(handle) => unsafe {
                handle.initialize(
                    NonNull::new(shm_ptr).ok_or(syscall::Error::new(syscall::EIO))?,
                    (),
                )
            },
            Err(_) => return Err(syscall::Error::new(syscall::EIO)),
        }
        RingPool::Dynamic(PoolDynamic {
            inner,
            shm_base: shm_ptr,
            total_size: params.pool_size as usize,
        })
    } else {
        RingPool::Chunk {
            allocator: ShmAllocator::new(params.pool_size as usize, chunk_size as usize),
            shm_base: shm_ptr,
        }
    };

    let pool_box = Box::new(pool {
        inner: ring_pool_inner,
    });

    let ring_ref = unsafe { ring.as_mut().ok_or(syscall::Error::new(syscall::EINVAL))? };
    ring_ref.sq = sq;
    ring_ref.cq = cq;
    ring_ref.pool = Box::into_raw(pool_box);
    ring_ref.ring_fd = ring_fd.take();

    Ok(())
}

pub unsafe fn redox_ring_alloc_buf(ring: *mut redox_ring, out_ptr: *mut *mut u8) -> c_int {
    let ring = unsafe { ring.as_mut() }.expect("redox_ring_alloc_buf: ring is null");
    let pool = unsafe { ring.pool.as_mut() }.expect("redox_ring_alloc_buf: ring pool is null");

    let chunk_size = match &pool.inner {
        RingPool::Chunk { allocator, .. } => allocator.chunk_size,
        RingPool::Dynamic(_) => DEFAULT_CHUNK_SIZE,
    };

    unsafe { redox_ring_alloc_buf_sized(ring, chunk_size as c_uint, 1 as c_uint, out_ptr) }
}

pub unsafe fn redox_ring_alloc_buf_sized(
    ring: *mut redox_ring,
    size: c_uint,
    align: c_uint,
    out_ptr: *mut *mut u8,
) -> c_int {
    let ring = unsafe { ring.as_mut() }.expect("redox_ring_alloc_buf_sized: ring is null");
    let pool =
        unsafe { ring.pool.as_mut() }.expect("redox_ring_alloc_buf_sized: ring pool is null");

    let ptr = pool.inner.allocate(size as usize, align as usize);
    if ptr.is_null() {
        return -1;
    }

    let out_ptr = unsafe { out_ptr.as_mut() }.expect("redox_ring_alloc_buf_sized: out is null");
    *out_ptr = ptr;

    let base = pool.inner.base_ptr();
    let offset = unsafe { ptr.offset_from(base) };
    offset as c_int
}

pub unsafe fn redox_ring_free_buf(ring: *mut redox_ring, ptr: *mut u8, offset: c_uint) -> c_int {
    let ring = unsafe { ring.as_mut() }.expect("redox_ring_free_buf: ring is null");
    let pool = unsafe { ring.pool.as_mut() }.expect("redox_ring_free_buf: ring pool is null");

    unsafe { pool.inner.deallocate(ptr, offset as usize) };
    0
}

pub unsafe fn redox_ring_get_sqe(ring: *mut redox_ring) -> *mut c_void {
    let ring = unsafe { ring.as_mut() }.expect("redox_ring_get_sqe: ring is null");

    match unsafe { ring.sq.get_sqe() } {
        Ok(sqe) => sqe.cast::<c_void>(),
        Err(_) => ptr::null_mut(),
    }
}

pub unsafe fn redox_ring_submit(ring: *mut redox_ring) -> c_int {
    let ring = unsafe { ring.as_mut() }.expect("redox_ring_submit: ring is null");

    ring.sq.submit()
}

pub unsafe fn redox_ring_wait_cqe(ring: *mut redox_ring, cqe: *mut c_void) -> c_int {
    let ring = unsafe { ring.as_mut() }.expect("redox_ring_wait_cqe: ring is null");
    if cqe.is_null() {
        panic!("redox_ring_wait_cqe: cqe is null");
    }
    let Ok(cqe_ptr) = ring.cq.peek_sync(None) else {
        return -1;
    };

    unsafe {
        ptr::copy_nonoverlapping(
            cqe_ptr.cast::<u8>(),
            cqe.cast::<u8>(),
            ring.cq.cqe_sz as usize,
        );
    }
    0
}

pub unsafe fn redox_ring_cqe_seen(ring: *mut redox_ring) {
    let ring = unsafe { ring.as_mut() }.expect("redox_ring_cqe_seen: ring is null");
    ring.cq.seen();
}

pub unsafe fn redox_ring_queue_exit(ring: *mut redox_ring) {
    let ring_ref = unsafe { ring.as_mut() }.expect("redox_ring_queue_exit: ring is null");
    if !ring_ref.pool.is_null() {
        let pool = unsafe { Box::from_raw(ring_ref.pool) };
        let _ = unsafe { syscall::funmap(pool.inner.base_ptr() as usize, pool.inner.total_size()) };
        ring_ref.pool = ptr::null_mut();
    }
    if !ring_ref.sq.ring_ptr.is_null() {
        let _ = unsafe { syscall::funmap(ring_ref.sq.ring_ptr as usize, ring_ref.sq.ring_sz) };
        ring_ref.sq.ring_ptr = ptr::null_mut();
    }
    if !ring_ref.cq.ring_ptr.is_null() {
        let _ = unsafe { syscall::funmap(ring_ref.cq.ring_ptr as usize, ring_ref.cq.ring_sz) };
        ring_ref.cq.ring_ptr = ptr::null_mut();
    }
    redox_rt::sys::close(ring_ref.ring_fd).unwrap();
}
