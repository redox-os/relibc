//! dl-tls implementation for Redox

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
    let mod_id = (*ti).ti_module as usize;
    let offset = if cfg!(target_arch = "riscv64") {
        ((*ti).ti_offset as usize).wrapping_add(0x800) // dynamic offsets are 0x800-based on risc-v
    } else {
        (*ti).ti_offset as usize
    };
    if mod_id > 0 {
        if let Some(tcb) = Tcb::current() {
            if let Some(masters) = tcb.masters() {
                if let Some(master) = masters.get(mod_id - 1) {
                    // module id is 1-based
                    let addr = if cfg!(any(target_arch = "x86", target_arch = "x86_64")) {
                        tcb.tls_end.sub(master.offset).add(offset)
                    } else {
                        // FIXME aarch64/risc-v only support static master
                        if mod_id == 1 && offset < tcb.tls_len {
                            tcb.tls_end.sub(tcb.tls_len).add(offset)
                        } else {
                            0 as *mut u8
                        }
                    };
                    if !addr.is_null() {
                        trace!(
                            "__tls_get_addr({:p}: {:#x}, {:#x}) = {:p}",
                            ti,
                            (*ti).ti_module,
                            (*ti).ti_offset,
                            addr
                        );
                        return addr as *mut c_void;
                    }
                }
            }
        }
    }
    panic!(
        "__tls_get_addr({:p}: {:#x}, {:#x}) failed",
        ti,
        (*ti).ti_module,
        (*ti).ti_offset
    );
}

// x86 can define a version that does not require stack alignment
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "C" fn ___tls_get_addr(ti: *mut dl_tls_index) -> *mut c_void {
    __tls_get_addr(ti)
}
