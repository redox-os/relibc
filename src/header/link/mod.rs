use crate::{
    ld_so::{dso::ProgramHeader, tcb::Tcb},
    platform::types::{c_char, c_int, c_ulonglong, c_void, size_t},
};

#[repr(C)]
pub struct dl_phdr_info {
    pub dlpi_addr: usize,
    pub dlpi_name: *const c_char,
    pub dlpi_phdr: *const ProgramHeader,
    pub dlpi_phnum: u16,
    pub dlpi_adds: c_ulonglong,
    pub dlpi_subs: c_ulonglong,
    pub dlpi_tls_modid: size_t,
    pub dlpi_tls_data: *const c_void,
}

pub type dl_iterate_phdr_callback =
    Option<unsafe extern "C" fn(info: *mut dl_phdr_info, size: usize, data: *mut c_void) -> c_int>;

/// Non-POSIX, see <https://man7.org/linux/man-pages/man3/dl_iterate_phdr.3.html>
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dl_iterate_phdr(
    callback: dl_iterate_phdr_callback,
    data: *mut c_void,
) -> c_int {
    let Some(tcb) = (unsafe { Tcb::current() }) else {
        return 0;
    };

    if tcb.linker_ptr.is_null() {
        return 0;
    }

    unsafe {
        let linker = (*tcb.linker_ptr).lock();
        linker.dl_iterate_phdr(callback, data)
    }
}
