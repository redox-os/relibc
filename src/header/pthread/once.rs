use crate::{header::pthread::pthread_once_t, platform::types::c_int};

// PTHREAD_ONCE_INIT

/// See <https://pubs.opengroup.org/onlinepubs/9799919799/functions/pthread_once.html>.
///
/// The first call to this function by any thread in a process, with a given
/// `once_control`, shall call the `init_routine` with no arguments. Subsequent
/// calls of this function with the same `once_control` shall not call the
/// `init_routine()`. The `once_control` parameter shall determine whether the
/// associated initialization routine has been called.
///
/// This function is not a cancellation point. However, if `init_routine()` is
/// a cancellation point and is canceled, the effect on `once_control` shall be
/// as if this function was never called.
///
/// Upon success, returns `0`. Upon failure, an error number is returned to
/// indicate the error.
///
/// # Implementation
/// Always succeeds so never returns an error number.
///
/// # Safety
/// It is undefined behaviour for the following:
/// - The call to `init_routine()` is terminated by a call to `longjmp()` or
///   `siglongjmp()`.
/// - If `once_control` has automatic storage duration or is not initialized by
///   `PTHREAD_ONCE_INIT`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn pthread_once(
    once_control: *mut pthread_once_t,
    init_routine: extern "C" fn(),
) -> c_int {
    let once = unsafe { &*once_control.cast::<RlctOnce>() };

    // TODO: Cancellation points

    once.call_once(|| init_routine());

    //crate::sync::once::call_once_generic(&once.inner, || init_routine());

    0
}
pub(crate) type RlctOnce = crate::sync::Once<()>;
