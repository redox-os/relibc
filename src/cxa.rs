use crate::{platform::types::*, raw_cell::RawCell};

// TODO: Implement cxa_finalize and uncomment this

#[derive(Clone, Copy)]
struct CxaAtExitFunc {
    //func: extern "C" fn(*mut c_void),
    //arg: *mut c_void,
    //dso: *mut c_void,
}

static CXA_ATEXIT_FUNCS: RawCell<[Option<CxaAtExitFunc>; 32]> = RawCell::new([None; 32]);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __cxa_atexit(
    func_opt: Option<extern "C" fn(*mut c_void)>,
    arg: *mut c_void,
    dso: *mut c_void,
) -> c_int {
    for i in 0..CXA_ATEXIT_FUNCS.unsafe_ref().len() {
        if CXA_ATEXIT_FUNCS.unsafe_ref()[i].is_none() {
            CXA_ATEXIT_FUNCS.unsafe_mut()[i] =
                func_opt.map(|func| CxaAtExitFunc {} /*{ func, arg, dso }*/);
            return 0;
        }
    }

    -1
}
