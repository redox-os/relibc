use super::*;

// PTHREAD_ONCE_INIT

#[no_mangle]
pub unsafe extern "C" fn pthread_once(once: *mut pthread_once_t, constructor: extern "C" fn()) -> c_int {
    let once: &pthread_once_t = &*once;

    // TODO: Cancellation points
    crate::sync::once::call_once_generic(&once.inner, || constructor());

    0
}
