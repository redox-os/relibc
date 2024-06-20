#![no_std]

use core::arch::asm;
use core::mem::{self, offset_of};

#[derive(Debug)]
#[repr(C)]
pub struct GenericTcb<Os> {
    /// Pointer to the end of static TLS. Must be the first member
    pub tls_end: *mut u8,
    /// Size of the memory allocated for the static TLS in bytes (multiple of page size)
    pub tls_len: usize,
    /// Pointer to this structure
    pub tcb_ptr: *mut Self,
    /// Size of the memory allocated for this structure in bytes (should be same as page size)
    pub tcb_len: usize,
    pub os_specific: Os,
}
impl<Os> GenericTcb<Os> {
    /// Architecture specific code to read a usize from the TCB - aarch64
    #[inline(always)]
    #[cfg(target_arch = "aarch64")]
    pub unsafe fn arch_read(offset: usize) -> usize {
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
    pub unsafe fn arch_read(offset: usize) -> usize {
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
    pub unsafe fn arch_read(offset: usize) -> usize {
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

    pub unsafe fn current_ptr() -> Option<*mut Self> {
        let tcb_ptr = Self::arch_read(offset_of!(Self, tcb_ptr)) as *mut Self;
        let tcb_len = Self::arch_read(offset_of!(Self, tcb_len));
        if tcb_ptr.is_null() || tcb_len < mem::size_of::<Self>() {
            None
        } else {
            Some(tcb_ptr)
        }
    }
}
