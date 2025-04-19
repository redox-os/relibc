use alloc::vec::Vec;
use core::{
    cell::UnsafeCell,
    mem,
    ops::{Deref, DerefMut},
    ptr, slice,
    sync::atomic::AtomicBool,
};
use generic_rt::GenericTcb;

use crate::{
    header::sys_mman,
    ld_so::linker::Linker,
    platform::{Dlmalloc, Pal, Sys},
    pthread::{OsTid, Pthread},
    sync::{mutex::Mutex, waitval::Waitval},
};

use super::linker::DlError;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Master {
    /// Pointer to initial data
    pub ptr: *const u8,
    /// Length of initial data in bytes
    pub len: usize,
    /// Offset in TLS to copy initial data to
    pub offset: usize,
}

impl Master {
    /// The initial data for this TLS region
    pub unsafe fn data(&self) -> &'static [u8] {
        slice::from_raw_parts(self.ptr, self.len)
    }
}

#[cfg(target_os = "linux")]
type OsSpecific = ();

#[cfg(target_os = "redox")]
type OsSpecific = redox_rt::signal::RtSigarea;

#[derive(Debug)]
#[repr(C)]
// FIXME: Only return &Tcb, and use interior mutability, since it contains the Pthread struct
pub struct Tcb {
    pub generic: GenericTcb<OsSpecific>,
    /// Pointer to a list of initial TLS data
    pub masters_ptr: *mut Master,
    /// Size of the masters list in bytes (multiple of mem::size_of::<Master>())
    pub masters_len: usize,
    /// Index of last copied Master
    pub num_copied_masters: usize,
    /// Pointer to dynamic linker
    pub linker_ptr: *const Mutex<Linker>,
    /// pointer to rust memory allocator structure
    pub mspace: *const Mutex<Dlmalloc>,
    /// Underlying pthread_t struct, pthread_self() returns &self.pthread
    pub pthread: Pthread,

    // Dynamic TLS Vector
    pub dtv_ptr: *mut *mut u8,
    // Number of DTV entries.
    pub dtv_len: usize,
}

#[cfg(target_os = "redox")]
const _: () = {
    if mem::size_of::<Tcb>() > syscall::PAGE_SIZE {
        panic!("too large TCB!");
    }
};

impl Tcb {
    /// Create a new TCB
    ///
    /// `size` is the size of the TLS in bytes.
    pub unsafe fn new(size: usize) -> Result<&'static mut Self, DlError> {
        let page_size = Sys::getpagesize();
        let (_abi_page, tls, tcb_page) = Self::os_new(size.next_multiple_of(page_size))?;

        let tcb_ptr = tcb_page.as_mut_ptr() as *mut Self;
        trace!("New TCB: {:p}", tcb_ptr);
        ptr::write(
            tcb_ptr,
            Self {
                generic: GenericTcb {
                    tls_end: tls.as_mut_ptr().add(tls.len()),
                    tls_len: tls.len(),
                    tcb_ptr: tcb_ptr.cast(),
                    tcb_len: tcb_page.len(),
                    os_specific: OsSpecific::default(),
                },
                masters_ptr: ptr::null_mut(),
                masters_len: 0,
                num_copied_masters: 0,
                linker_ptr: ptr::null(),
                mspace: ptr::null(),
                pthread: Pthread {
                    waitval: Waitval::new(),
                    flags: Default::default(),
                    has_enabled_cancelation: AtomicBool::new(false),
                    has_queued_cancelation: AtomicBool::new(false),
                    stack_base: core::ptr::null_mut(),
                    stack_size: 0,
                    os_tid: UnsafeCell::new(OsTid::default()),
                },

                dtv_ptr: ptr::null_mut(),
                dtv_len: 0,
            },
        );

        Ok(&mut *tcb_ptr)
    }

    /// Get the current TCB
    pub unsafe fn current() -> Option<&'static mut Self> {
        Some(&mut *GenericTcb::<OsSpecific>::current_ptr()?.cast())
    }

    /// A slice for all of the TLS data
    pub unsafe fn tls(&self) -> Option<&'static mut [u8]> {
        if self.tls_end.is_null() || self.tls_len == 0 {
            None
        } else {
            let tls_start = self.tls_end.sub(self.tls_len);
            Some(slice::from_raw_parts_mut(tls_start, self.tls_len))
        }
    }

    /// The initial images for TLS
    pub fn masters(&self) -> Option<&'static mut [Master]> {
        if self.masters_ptr.is_null() || self.masters_len == 0 {
            None
        } else {
            Some(unsafe {
                slice::from_raw_parts_mut(
                    self.masters_ptr,
                    self.masters_len / mem::size_of::<Master>(),
                )
            })
        }
    }

    /// Copy data from masters
    pub unsafe fn copy_masters(&mut self) -> Result<(), DlError> {
        //TODO: Complain if masters or tls exist without the other
        if let Some(tls) = self.tls() {
            if let Some(masters) = self.masters() {
                for master in masters
                    .iter()
                    .skip(self.num_copied_masters)
                    .filter(|master| master.len != 0)
                {
                    let range = if cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
                        // x86{_64} TLS layout is backwards
                        self.tls_len - master.offset..self.tls_len - master.offset + master.len
                    } else {
                        master.offset..master.offset + master.len
                    };
                    if let Some(tls_data) = tls.get_mut(range) {
                        let data = master.data();
                        trace!(
                            "tls master {}: {:p}, {:#x}: {:p}, {:#x}",
                            i,
                            data.as_ptr(),
                            data.len(),
                            tls_data.as_mut_ptr(),
                            tls_data.len()
                        );
                        tls_data.copy_from_slice(data);
                    } else {
                        return Err(DlError::Malformed);
                    }
                }
                self.num_copied_masters = masters.len();
            }
        }

        Ok(())
    }

    /// The initial images for TLS
    pub unsafe fn append_masters(&mut self, mut new_masters: Vec<Master>) {
        if self.masters_ptr.is_null() {
            self.masters_ptr = new_masters.as_mut_ptr();
            self.masters_len = new_masters.len() * mem::size_of::<Master>();
            mem::forget(new_masters);
        } else {
            // XXX: [`Vec::from_raw_parts`] cannot be used here as the masters were originally
            // allocated by the ld.so allocator and that would violate that function's invariants.
            let mut masters = self.masters().unwrap().to_vec();
            masters.extend(new_masters);

            self.masters_ptr = masters.as_mut_ptr();
            self.masters_len = masters.len() * mem::size_of::<Master>();
            mem::forget(masters);
        }
    }

    /// Activate TLS
    pub unsafe fn activate(&mut self, #[cfg(target_os = "redox")] thr_fd: redox_rt::proc::FdGuard) {
        Self::os_arch_activate(
            &self.os_specific,
            self.tls_end as usize,
            self.tls_len,
            #[cfg(target_os = "redox")]
            thr_fd,
        );
    }

    pub fn setup_dtv(&mut self, n: usize) {
        if self.dtv_ptr.is_null() {
            let mut dtv = vec![ptr::null_mut(); n];

            if let Some(masters) = self.masters() {
                for (i, master) in masters.iter().enumerate() {
                    let tls = unsafe { self.tls().unwrap() };
                    let offset = if cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
                        // x86{_64} TLS layout is backwards
                        self.tls_len - master.offset
                    } else {
                        master.offset
                    };

                    dtv[i] = unsafe { tls.as_mut_ptr().add(offset) };
                }
            }

            let (ptr, len, _) = dtv.into_raw_parts();

            self.dtv_ptr = ptr;
            self.dtv_len = len;
        } else {
            // Resize DTV.
            //
            // XXX: [`Vec::from_raw_parts`] cannot be used here as the DTV was originally allocated
            // by the ld.so allocator and that would violate that function's invariants.
            let mut dtv = self.dtv_mut().to_vec();
            dtv.resize(n, ptr::null_mut());

            let (ptr, len, _) = dtv.into_raw_parts();
            self.dtv_ptr = ptr;
            self.dtv_len = len;
        }
    }

    pub fn dtv_mut(&mut self) -> &'static mut [*mut u8] {
        if self.dtv_len != 0 {
            unsafe { slice::from_raw_parts_mut(self.dtv_ptr, self.dtv_len) }
        } else {
            &mut []
        }
    }

    /// Mapping with correct flags for TCB and TLS
    unsafe fn map(size: usize) -> Result<&'static mut [u8], DlError> {
        let ptr = Sys::mmap(
            ptr::null_mut(),
            size,
            sys_mman::PROT_READ | sys_mman::PROT_WRITE,
            sys_mman::MAP_ANONYMOUS | sys_mman::MAP_PRIVATE,
            -1,
            0,
        )
        .map_err(|_| DlError::Oom)?;

        ptr::write_bytes(ptr as *mut u8, 0, size);
        Ok(slice::from_raw_parts_mut(ptr as *mut u8, size))
    }

    /// OS specific code to create a new TLS and TCB - Linux and Redox
    ///
    /// Memory layout:
    ///
    /// ```text
    /// 0          page_size                   size       (size + page_size * 2)
    /// |----------|---------------------------|----------|
    /// +++++++++++++++++++++++++++++++++++++++++++++++++++
    /// | ABI Page | TLS                       | TCB Page |
    /// +++++++++++++++++++++++++++++++++++++++++++++++++++
    ///     ^ $tp (aarch64)                    ^ $tp (x86_64)
    /// ```
    ///
    /// `$tp` refers to the architecture specific thread pointer.
    ///
    /// **Note**: On x86{_64}, the TLS layout is backwards (i.e. the first byte of the TLS is at
    /// the end of the TLS region).
    ///
    /// ABI page layout for aarch64:
    /// ```text
    /// 0                     4096
    /// +---------------------+
    /// | ABI Page            |
    /// +---------------------+
    ///                     ^
    ///                     |
    ///                     +-------> (page_size - 16): pointer to the start of the TCB page
    /// ```
    ///
    /// ABI page layout for riscv64:
    ///
    /// ```text
    /// 0                     4096
    /// +---------------------+
    /// | ABI Page            |
    /// +---------------------+
    ///                      ^
    ///                      |
    ///                      +-------> (page_size - 8): pointer to the start of the TCB page
    /// ```
    ///
    /// For x86_64, the ABI page is not used.
    #[cfg(any(target_os = "linux", target_os = "redox"))]
    unsafe fn os_new(
        size: usize,
    ) -> Result<(&'static mut [u8], &'static mut [u8], &'static mut [u8]), DlError> {
        let page_size = Sys::getpagesize();
        let abi_tls_tcb = Self::map(page_size + size + page_size)?;
        let (abi, tls_tcb) = abi_tls_tcb.split_at_mut(page_size);
        let (tls, tcb) = tls_tcb.split_at_mut(size);
        Ok((abi, tls, tcb))
    }

    /// OS and architecture specific code to activate TLS - Linux x86_64
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    unsafe fn os_arch_activate(_os: &(), tls_end: usize, _tls_len: usize) {
        const ARCH_SET_FS: usize = 0x1002;
        syscall!(ARCH_PRCTL, ARCH_SET_FS, tls_end);
    }

    #[cfg(target_os = "redox")]
    unsafe fn os_arch_activate(
        os: &OsSpecific,
        tls_end: usize,
        tls_len: usize,
        thr_fd: redox_rt::proc::FdGuard,
    ) {
        os.thr_fd.get().write(Some(thr_fd));
        redox_rt::tcb_activate(os, tls_end, tls_len)
    }
}

impl Deref for Tcb {
    type Target = GenericTcb<OsSpecific>;

    fn deref(&self) -> &Self::Target {
        &self.generic
    }
}
impl DerefMut for Tcb {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.generic
    }
}
