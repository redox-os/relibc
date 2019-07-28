use crate::platform::types::*;

// TODO: Implement cxa_finalize and uncomment this

#[derive(Clone, Copy)]
struct CxaAtExitFunc {
    //func: extern "C" fn(*mut c_void),
//arg: *mut c_void,
//dso: *mut c_void,
}

static mut CXA_ATEXIT_FUNCS: [Option<CxaAtExitFunc>; 32] = [None; 32];

#[no_mangle]
pub unsafe extern "C" fn __cxa_atexit(
    func_opt: Option<extern "C" fn(*mut c_void)>,
    arg: *mut c_void,
    dso: *mut c_void,
) -> c_int {
    for item in &mut CXA_ATEXIT_FUNCS {
        if item.is_none() {
            *item = func_opt.map(|func| CxaAtExitFunc {} /*{ func, arg, dso }*/);
            return 0;
        }
    }

    -1
}
