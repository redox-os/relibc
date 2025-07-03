//! dl-tls implementation for Redox

#![deny(unsafe_op_in_unsafe_fn)]

use core::{alloc::Layout, arch::global_asm};

use alloc::alloc::alloc_zeroed;

use crate::{ld_so::tcb::Tcb, platform::types::*};

#[repr(C)]
pub struct dl_tls_index {
    pub ti_module: usize,
    pub ti_offset: usize,
}

#[no_mangle]
pub unsafe extern "C" fn __tls_get_addr(ti: *mut dl_tls_index) -> *mut c_void {
    let tcb = unsafe { Tcb::current().unwrap() };
    let ti = unsafe { &*ti };
    let masters = unsafe { tcb.masters().unwrap() };

    trace!(
        "__tls_get_addr({:p}: {:#x}, {:#x}, masters_len={}, dtv_len={})",
        ti,
        ti.ti_module,
        ti.ti_offset,
        masters.len(),
        tcb.dtv_mut().len()
    );

    if tcb.dtv_mut().len() < masters.len() {
        // Reallocate DTV.
        tcb.setup_dtv(masters.len());
    }

    let dtv_index = ti.ti_module as usize - 1;

    if tcb.dtv_mut()[dtv_index].is_null() {
        // Allocate TLS for module.
        let master = &masters[dtv_index];

        // FIXME(andypython): master.align
        let layout = unsafe {
            Layout::from_size_align_unchecked(master.offset /* aligned ph.p_memsz */, 16)
        };

        let module_tls = unsafe { alloc_zeroed(layout) };

        unsafe { core::ptr::copy_nonoverlapping(master.ptr, module_tls, master.len) };

        // Set the DTV entry.
        tcb.dtv_mut()[dtv_index] = module_tls;
    }

    let mut ptr = tcb.dtv_mut()[dtv_index];

    if ptr.is_null() {
        panic!(
            "__tls_get_addr({ti:p}: {:#x}, {:#x})",
            ti.ti_module, ti.ti_offset
        );
    }

    if cfg!(target_arch = "riscv64") {
        ptr = unsafe { ptr.add(0x800 + ti.ti_offset as usize) }; // dynamic offsets are 0x800-based on risc-v
    } else {
        ptr = unsafe { ptr.add(ti.ti_offset as usize) };
    }

    ptr.cast::<c_void>()
}

// x86 can define a version that passes a pointer to dl_tls_index in eax
#[cfg(target_arch = "x86")]
global_asm!(
    "
    .globl ___tls_get_addr
    .type ___tls_get_addr, @function
___tls_get_addr:
    push ebp
    mov ebp, esp
    push eax
    call __tls_get_addr
    add esp, 4
    pop ebp
    ret
    .size ___tls_get_addr, . - ___tls_get_addr
"
);
