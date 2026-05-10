use super::*;

// PTHREAD_ONCE_INIT

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_once.html>.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_once(
    once: *mut pthread_once_t,
    constructor: extern "C" fn(),
) -> c_int {
    let once = unsafe { &*once.cast::<RlctOnce>() };

    // TODO: Cancellation points

    once.call_once(|| constructor());

    //crate::sync::once::call_once_generic(&once.inner, || constructor());

    0
}
pub(crate) type RlctOnce = crate::sync::Once<()>;
