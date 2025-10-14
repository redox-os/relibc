use crate::{
    header::time::timer_internal_t,
    platform::{sys::libredox, types::c_void},
};
use core::ptr;

pub extern "C" fn timer_routine(arg: *mut c_void) -> *mut c_void {
    unsafe {
        let timer = &mut *(arg as *mut timer_internal_t);

        loop {
            let mut buf = [0u8; 8];
            if Sys::read(timer.eventfd, &mut buf).is_err() {
                break;
            }

            if !timer.evp.sigev_notify_function.is_null() {
                timer.evp.sigev_notify_function(timer.evp.sigev_value);
            }
        }
    }

    ptr::null_mut()
}
