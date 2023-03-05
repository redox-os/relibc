use super::*;

#[repr(C)]
pub struct Once {
    inner: crate::sync::Once<()>,
}

// PTHREAD_ONCE_INIT

#[no_mangle]
pub unsafe extern "C" fn pthread_once(once: *mut pthread_once_t, constructor: extern "C" fn()) -> c_int {
    let once: &pthread_once_t = &*once;

    // TODO: Cancellation points
    once.inner.call_once(|| constructor());

    0
}
