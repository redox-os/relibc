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
    if let Some(tcb) = Tcb::current() {
        if let Some(masters) = tcb.masters() {
            if let Some(master) = masters.get((*ti).ti_module as usize) {
                let addr = tcb.tls_end.sub(master.offset).add((*ti).ti_offset as usize);
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
    panic!(
        "__tls_get_addr({:p}: {:#x}, {:#x}) failed",
        ti,
        (*ti).ti_module,
        (*ti).ti_offset
    );
}
