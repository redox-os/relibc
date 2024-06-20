use alloc::vec::Vec;
use syscall::Sigcontrol;
use core::{arch::asm, cell::UnsafeCell, mem::{self, offset_of}, ptr, slice, sync::atomic::AtomicBool};
use goblin::error::{Error, Result};

use super::ExpectTlsFree;
use crate::{
    header::sys_mman,
    ld_so::linker::Linker,
    platform::{Dlmalloc, Pal, Sys},
    pthread::{OsTid, Pthread},
    sync::{mutex::Mutex, waitval::Waitval},
};

#[repr(C)]
#[derive(Debug)]
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

#[derive(Debug)]
#[repr(C)]
// FIXME: Only return &Tcb, and use interior mutability, since it contains the Pthread struct
pub struct Tcb {
    /// Pointer to the end of static TLS. Must be the first member
    pub tls_end: *mut u8,
    /// Size of the memory allocated for the static TLS in bytes (multiple of page size)
    pub tls_len: usize,
    /// Pointer to this structure
    pub tcb_ptr: *mut Tcb,
    /// Size of the memory allocated for this structure in bytes (should be same as page size)
    pub tcb_len: usize,
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
    #[cfg(target_os = "redox")]
    pub sigcontrol: Sigcontrol,
}

impl Tcb {
    /// Create a new TCB
    pub unsafe fn new(size: usize) -> Result<&'static mut Self> {
        let page_size = Sys::getpagesize();
        let (abi_page, tls, tcb_page) = Self::os_new(round_up(size, page_size))?;

        let tcb_ptr = tcb_page.as_mut_ptr() as *mut Self;
        trace!("New TCB: {:p}", tcb_ptr);
        ptr::write(
            tcb_ptr,
            Self {
                tls_end: tls.as_mut_ptr().add(tls.len()),
                tls_len: tls.len(),
                tcb_ptr,
                tcb_len: tcb_page.len(),
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
                #[cfg(target_os = "redox")]
                sigcontrol: Default::default(),
            },
        );

        Ok(&mut *tcb_ptr)
    }

    /// Get the current TCB
    pub unsafe fn current() -> Option<&'static mut Self> {
        let tcb_ptr = Self::arch_read(offset_of!(Self, tcb_ptr)) as *mut Self;
        let tcb_len = Self::arch_read(offset_of!(Self, tcb_len));
        if tcb_ptr.is_null() || tcb_len < mem::size_of::<Self>() {
            None
        } else {
            Some(&mut *tcb_ptr)
        }
    }

    /// A slice for all of the TLS data
    pub unsafe fn tls(&self) -> Option<&'static mut [u8]> {
        if self.tls_end.is_null() || self.tls_len == 0 {
            None
        } else {
            Some(slice::from_raw_parts_mut(
                self.tls_end.offset(-(self.tls_len as isize)),
                self.tls_len,
            ))
        }
    }

    /// The initial images for TLS
    pub unsafe fn masters(&self) -> Option<&'static mut [Master]> {
        if self.masters_ptr.is_null() || self.masters_len == 0 {
            None
        } else {
            Some(slice::from_raw_parts_mut(
                self.masters_ptr,
                self.masters_len / mem::size_of::<Master>(),
            ))
        }
    }

    /// Copy data from masters
    pub unsafe fn copy_masters(&mut self) -> Result<()> {
        //TODO: Complain if masters or tls exist without the other
        if let Some(tls) = self.tls() {
            if let Some(masters) = self.masters() {
                for (i, master) in masters
                    .iter()
                    .skip(self.num_copied_masters)
                    .filter(|m| m.len > 0)
                    .enumerate()
                {
                    let range = if cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
                        // x86 TLS layout is backwards
                        self.tls_len - master.offset..self.tls_len - master.offset + master.len
                    } else {
                        //TODO: fix aarch64 TLS layout when there is more than one master
                        assert_eq!(i, 0, "aarch64 TLS layout only supports one master");
                        0..master.len
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
                        return Err(Error::Malformed(format!("failed to copy tls master {}", i)));
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
            let len = self.masters_len / mem::size_of::<Master>();
            let mut masters = Vec::from_raw_parts(self.masters_ptr, len, len);
            masters.extend(new_masters.into_iter());
            self.masters_ptr = masters.as_mut_ptr();
            self.masters_len = masters.len() * mem::size_of::<Master>();
            mem::forget(masters);
        }
    }

    /// Activate TLS
    pub unsafe fn activate(&mut self) {
        Self::os_arch_activate(self.tls_end as usize, self.tls_len);
    }

    /// Mapping with correct flags for TCB and TLS
    unsafe fn map(size: usize) -> Result<&'static mut [u8]> {
        let ptr = sys_mman::mmap(
            ptr::null_mut(),
            size,
            sys_mman::PROT_READ | sys_mman::PROT_WRITE,
            sys_mman::MAP_ANONYMOUS | sys_mman::MAP_PRIVATE,
            -1,
            0,
        );
        if ptr as usize == !0
        /* MAP_FAILED */
        {
            return Err(Error::Malformed(format!("failed to map tls")));
        }
        ptr::write_bytes(ptr as *mut u8, 0, size);
        Ok(slice::from_raw_parts_mut(ptr as *mut u8, size))
    }

    /// OS specific code to create a new TLS and TCB - Linux and Redox
    #[cfg(any(target_os = "linux", target_os = "redox"))]
    unsafe fn os_new(
        size: usize,
    ) -> Result<(&'static mut [u8], &'static mut [u8], &'static mut [u8])> {
        let page_size = Sys::getpagesize();
        let abi_tls_tcb = Self::map(page_size + size + page_size)?;
        let (abi, tls_tcb) = abi_tls_tcb.split_at_mut(page_size);
        let (tls, tcb) = tls_tcb.split_at_mut(size);
        Ok((abi, tls, tcb))
    }

    /// Architecture specific code to read a usize from the TCB - aarch64
    #[inline(always)]
    #[cfg(target_arch = "aarch64")]
    unsafe fn arch_read(offset: usize) -> usize {
        let abi_ptr: usize;
        asm!(
            "mrs {}, tpidr_el0",
            out(reg) abi_ptr,
        );

        let tcb_ptr = *(abi_ptr as *const usize);
        *((tcb_ptr + offset) as *const usize)
    }

    /// Architecture specific code to read a usize from the TCB - x86
    #[inline(always)]
    #[cfg(target_arch = "x86")]
    unsafe fn arch_read(offset: usize) -> usize {
        let value;
        asm!(
            "
            mov {}, gs:[{}]
            ",
            out(reg) value,
            in(reg) offset,
        );
        value
    }

    /// Architecture specific code to read a usize from the TCB - x86_64
    #[inline(always)]
    #[cfg(target_arch = "x86_64")]
    unsafe fn arch_read(offset: usize) -> usize {
        let value;
        asm!(
            "
            mov {}, fs:[{}]
            ",
            out(reg) value,
            in(reg) offset,
        );
        value
    }

    /// OS and architecture specific code to activate TLS - Linux x86_64
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    unsafe fn os_arch_activate(tls_end: usize, _tls_len: usize) {
        const ARCH_SET_FS: usize = 0x1002;
        syscall!(ARCH_PRCTL, ARCH_SET_FS, tls_end);
    }

    /// OS and architecture specific code to activate TLS - Redox aarch64
    #[cfg(all(target_os = "redox", target_arch = "aarch64"))]
    unsafe fn os_arch_activate(tls_end: usize, tls_len: usize) {
        // Uses ABI page
        let abi_ptr = tls_end - tls_len - 16;
        ptr::write(abi_ptr as *mut usize, tls_end);
        asm!(
            "msr tpidr_el0, {}",
            in(reg) abi_ptr,
        );
    }

    /// OS and architecture specific code to activate TLS - Redox x86
    #[cfg(all(target_os = "redox", target_arch = "x86"))]
    unsafe fn os_arch_activate(tls_end: usize, _tls_len: usize) {
        let mut env = syscall::EnvRegisters::default();

        let file = syscall::open(
            "thisproc:current/regs/env",
            syscall::O_CLOEXEC | syscall::O_RDWR,
        )
        .expect_notls("failed to open handle for process registers");

        let _ = syscall::read(file, &mut env).expect_notls("failed to read gsbase");

        env.gsbase = tls_end as u32;

        let _ = syscall::write(file, &env).expect_notls("failed to write gsbase");

        let _ = syscall::close(file);
    }

    /// OS and architecture specific code to activate TLS - Redox x86_64
    #[cfg(all(target_os = "redox", target_arch = "x86_64"))]
    unsafe fn os_arch_activate(tls_end: usize, _tls_len: usize) {
        let mut env = syscall::EnvRegisters::default();

        let file = syscall::open(
            "thisproc:current/regs/env",
            syscall::O_CLOEXEC | syscall::O_RDWR,
        )
        .expect_notls("failed to open handle for process registers");

        let _ = syscall::read(file, &mut env).expect_notls("failed to read fsbase");

        env.fsbase = tls_end as u64;

        let _ = syscall::write(file, &env).expect_notls("failed to write fsbase");

        let _ = syscall::close(file);
    }
}

pub fn round_up(value: usize, alignment: usize) -> usize {
    return (value + alignment - 1) & (!(alignment - 1));
}
