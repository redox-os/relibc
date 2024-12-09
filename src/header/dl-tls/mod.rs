//! dl-tls implementation for Redox

#![deny(unsafe_op_in_unsafe_fn)]

use crate::{ld_so::tcb::Tcb, platform::types::*};

#[repr(C)]
pub struct dl_tls_index {
    pub ti_module: u64,
    pub ti_offset: u64,
}

#[no_mangle]
pub unsafe extern "C" fn __tls_get_addr(ti: *mut dl_tls_index) -> *mut c_void {
    trace!(
        "__tls_get_addr({:p}: {:#x}, {:#x})",
        ti,
        (*ti).ti_module,
        (*ti).ti_offset
    );
    let mod_id = unsafe { (*ti).ti_module } as usize;
    let offset = {
        let ti_offset = unsafe { (*ti).ti_offset } as usize;
        if cfg!(target_arch = "riscv64") {
            ti_offset.wrapping_add(0x800) // dynamic offsets are 0x800-based on risc-v
        } else {
            ti_offset
        }
    };
    if mod_id > 0 {
        if let Some(tcb) = unsafe { Tcb::current() } {
            if let Some(masters) = unsafe { tcb.masters() } {
                if let Some(master) = masters.get(mod_id - 1) {
                    // module id is 1-based
                    let addr = if cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
                        unsafe { tcb.tls_end.sub(master.offset).add(offset) }
                    } else {
                        // FIXME aarch64/risc-v only support static master
                        if mod_id == 1 && offset < tcb.tls_len {
                            unsafe { tcb.tls_end.sub(tcb.tls_len).add(offset) }
                        } else {
                            0 as *mut u8
                        }
                    };
                    if !addr.is_null() {
                        let (ti_module, ti_offset) = unsafe { ((*ti).ti_module, (*ti).ti_offset) };
                        trace!(
                            "__tls_get_addr({:p}: {:#x}, {:#x}) = {:p}",
                            ti,
                            ti_module,
                            ti_offset,
                            addr
                        );
                        return addr as *mut c_void;
                    }
                }
            }
        }
    }
    let (ti_module, ti_offset) = unsafe { ((*ti).ti_module, (*ti).ti_offset) };
    panic!(
        "__tls_get_addr({:p}: {:#x}, {:#x}) failed",
        ti, ti_module, ti_offset
    );
}

// x86 can define a version that does not require stack alignment
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "C" fn ___tls_get_addr(ti: *mut dl_tls_index) -> *mut c_void {
    unsafe { __tls_get_addr(ti) }
}
