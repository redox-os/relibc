use platform::types::*;

#[derive(Clone, Copy)]
struct CxaAtExitFunc {
    func: extern "C" fn(*mut c_void),
    arg: *mut c_void,
    dso: *mut c_void,
}

static mut CXA_ATEXIT_FUNCS: [Option<CxaAtExitFunc>; 32] = [None; 32];

#[no_mangle]
pub unsafe extern "C" fn __cxa_atexit(
    func_opt: Option<extern "C" fn(*mut c_void)>,
    arg: *mut c_void,
    dso: *mut c_void,
) -> c_int {
    for i in 0..CXA_ATEXIT_FUNCS.len() {
        if CXA_ATEXIT_FUNCS[i].is_none() {
            CXA_ATEXIT_FUNCS[i] = func_opt.map(|func| CxaAtExitFunc { func, arg, dso });
            return 0;
        }
    }

    -1
}

// TODO: cxa_finalize
