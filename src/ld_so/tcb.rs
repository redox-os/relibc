use alloc::vec::Vec;
use core::{mem, ptr, slice};
use goblin::error::{Error, Result};

use crate::{header::sys_mman, ld_so::linker::Linker, sync::mutex::Mutex};

use super::PAGE_SIZE;

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
    /// Size of the memory allocated for the static TLS in bytes (multiple of PAGE_SIZE)
    pub tls_len: usize,
    /// Pointer to this structure
    pub tcb_ptr: *mut Tcb,
    /// Size of the memory allocated for this structure in bytes (should be PAGE_SIZE)
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
        let (tls, tcb_page) = Self::os_new(round_up(size, PAGE_SIZE))?;

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

    /// OS specific code to create a new TLS and TCB - Linux
    #[cfg(target_os = "linux")]
    unsafe fn os_new(size: usize) -> Result<(&'static mut [u8], &'static mut [u8])> {
        let tls_tcb = Self::map(size + PAGE_SIZE)?;
        Ok(tls_tcb.split_at_mut(size))
    }

    /// OS specific code to create a new TLS and TCB - Redox
    #[cfg(target_os = "redox")]
    unsafe fn os_new(size: usize) -> Result<(&'static mut [u8], &'static mut [u8])> {
        use crate::header::unistd;
        //TODO: better method of finding fs offset
        let pid = unistd::getpid();
        let tcb_addr = 0xB000_0000 + pid as usize * PAGE_SIZE;
        let tls = Self::map(size)?;
        Ok((
            tls,
            //TODO: Consider allocating TCB as part of TLS
            slice::from_raw_parts_mut(tcb_addr as *mut u8, PAGE_SIZE),
        ))
    }

    /// Architecture specific code to read a usize from the TCB - x86_64
    #[inline(always)]
    #[cfg(target_arch = "aarch64")]
    unsafe fn arch_read(offset: usize) -> usize {
        //TODO: aarch64
        unimplemented!("arch_read not implemented on aarch64");
    }

    /// Architecture specific code to read a usize from the TCB - x86_64
    #[inline(always)]
    #[cfg(target_arch = "x86_64")]
    unsafe fn arch_read(offset: usize) -> usize {
        let value;
        llvm_asm!("
            mov rax, fs:[rdi]
            "
            : "={rax}"(value)
            : "{rdi}"(offset)
            :
            : "intel"
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
        //TODO: Consider setting FS offset to TCB pointer
    }
}

pub fn round_up(value: usize, alignment: usize) -> usize {
    return (value + alignment - 1) & (!(alignment - 1));
}
