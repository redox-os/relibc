use super::*;

// PTHREAD_ONCE_INIT

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_once(
    once: *mut pthread_once_t,
    constructor: extern "C" fn(),
) -> c_int {
    let once = &*once.cast::<RlctOnce>();

    // TODO: Cancellation points

    once.call_once(|| constructor());

    //crate::sync::once::call_once_generic(&once.inner, || constructor());

    0
}
pub(crate) type RlctOnce = crate::sync::Once<()>;
