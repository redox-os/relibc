use core::{mem, ptr, slice};
use core::arch::asm;
use alloc::vec::Vec;
use goblin::error::{Error, Result};

use crate::{
    header::sys_mman,
    ld_so::{linker::Linker, ExpectTlsFree},
    platform::{Pal, Sys},
    sync::mutex::Mutex,
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
    pub mspace: usize,
}

impl Tcb {
    /// Create a new TCB
    pub unsafe fn new(size: usize) -> Result<&'static mut Self> {
        let page_size = Sys::getpagesize();
        let (tls, tcb_page) = Self::os_new(round_up(size, page_size))?;

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
                mspace: 0,
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
                    let range =
                        self.tls_len - master.offset..self.tls_len - master.offset + master.len;
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
        Self::os_arch_activate(self.tcb_ptr as usize);
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
    unsafe fn os_new(size: usize) -> Result<(&'static mut [u8], &'static mut [u8])> {
        let page_size = Sys::getpagesize();
        let tls_tcb = Self::map(size + page_size)?;
        Ok(tls_tcb.split_at_mut(size))
    }

    /// Architecture specific code to read a usize from the TCB - x86_64
    #[inline(always)]
    #[cfg(target_arch = "aarch64")]
    unsafe fn arch_read(offset: usize) -> usize {
        // TODO: s/llvm_asm/asm/g
        let tp: usize;
        llvm_asm!("mrs $0, tpidr_el0"
            : "=r"(tp)
            :
            :
            : "volatile"
        );

        *((tp + offset) as *const usize)
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
    unsafe fn os_arch_activate(tp: usize) {
        const ARCH_SET_FS: usize = 0x1002;
        syscall!(ARCH_PRCTL, ARCH_SET_FS, tp);
    }

    /// OS and architecture specific code to activate TLS - Redox aarch64
    #[cfg(all(target_os = "redox", target_arch = "aarch64"))]
    unsafe fn os_arch_activate(tp: usize) {
        //TODO: aarch64
    }

    /// OS and architecture specific code to activate TLS - Redox x86_64
    #[cfg(all(target_os = "redox", target_arch = "x86_64"))]
    unsafe fn os_arch_activate(tp: usize) {
        let mut env = syscall::EnvRegisters::default();

        let file = syscall::open("thisproc:current/regs/env", syscall::O_CLOEXEC | syscall::O_RDWR)
            .expect_notls("failed to open handle for process registers");

        let _ = syscall::read(file, &mut env)
            .expect_notls("failed to read fsbase");

        env.fsbase = tp as u64;

        let _ = syscall::write(file, &env)
            .expect_notls("failed to write fsbase");

        let _ = syscall::close(file);
    }
}

pub fn round_up(value: usize, alignment: usize) -> usize {
    return (value + alignment - 1) & (!(alignment - 1));
}
