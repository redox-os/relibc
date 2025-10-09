//! sys/select.h implementation

use core::{mem, ptr};

use cbitset::BitSet;

use crate::{
    error::{Errno, Result, ResultExt},
    fs::File,
    header::{
        errno,
        signal::{SIG_SETMASK, sigprocmask, sigset_t},
        sys_epoll::{
            EPOLL_CLOEXEC, EPOLL_CTL_ADD, EPOLLERR, EPOLLIN, EPOLLOUT, epoll_create1, epoll_ctl,
            epoll_data, epoll_event, epoll_wait,
        },
        sys_time::timeval,
        time::timespec,
    },
    platform::{self, Pal, Sys, types::*},
};

// fd_set is also defined in C because cbindgen is incompatible with mem::size_of booo

pub const FD_SETSIZE: usize = 1024;
pub type bitset = BitSet<[u64; FD_SETSIZE / (8 * mem::size_of::<u64>())]>;

#[repr(C)]
pub struct fd_set {
    pub fds_bits: bitset,
}

#[unsafe(no_mangle)]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe extern "C" fn select(
    nfds: c_int,
    readfds: *mut fd_set,
    writefds: *mut fd_set,
    exceptfds: *mut fd_set,
    timeout: *mut timeval,
) -> c_int {
    let Ok(nfds) = nfds.try_into() else {
        return Err(Errno(errno::EINVAL)).or_minus_one_errno();
    };

    // select's timeout is a timeval whereas pselect's is a timespec.
    // Like Linux, our pselect modifies the timespec with the remaining time.
    let mut timeout = (!timeout.is_null()).then(|| &mut *timeout);
    let mut timespec: Option<timespec> = timeout.as_ref().map(|&&mut timeval| timeval.into());

    let result = trace_expr!(
        Sys::pselect(
            nfds,
            if readfds.is_null() {
                None
            } else {
                Some(&mut *readfds)
            },
            if writefds.is_null() {
                None
            } else {
                Some(&mut *writefds)
            },
            if exceptfds.is_null() {
                None
            } else {
                Some(&mut *exceptfds)
            },
            timespec.as_mut(),
            None
        )
        .map(|fds| fds
            .try_into()
            .expect("nfds is bound between [0, {FD_SETSIZE}]"))
        .or_minus_one_errno(),
        "select({}, {:p}, {:p}, {:p}, {:?})",
        nfds,
        readfds,
        writefds,
        exceptfds,
        timeout
    );

    // Update remaining time
    if result != 0
        && let Some(timeout) = timeout
        && let Some(timespec) = timespec
    {
        *timeout = timespec.into();
    }

    result
}

#[unsafe(no_mangle)]
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn pselect(
    nfds: c_int,
    readfds: *mut fd_set,
    writefds: *mut fd_set,
    exceptfds: *mut fd_set,
    timeout: *const timespec,
    sigmask: *const sigset_t,
) -> c_int {
    let Ok(nfds) = nfds.try_into() else {
        return Err(Errno(errno::EINVAL)).or_minus_one_errno();
    };

    // POSIX compatible pselect doesn't modify the timeout.
    // Redox's pselect, like Linux's and others', modifies the timeout so we have to make sure to
    // be const-safe and not clobber timeout.
    let mut timeout = (!timeout.is_null()).then(|| *timeout);

    // Null pointers are valid for both pselect and sigprocmask.
    // A null sigmask means pselect acts like select.
    let sigmask = (!sigmask.is_null()).then(|| *sigmask);

    trace_expr!(
        Sys::pselect(
            nfds,
            if readfds.is_null() {
                None
            } else {
                Some(&mut *readfds)
            },
            if writefds.is_null() {
                None
            } else {
                Some(&mut *writefds)
            },
            if exceptfds.is_null() {
                None
            } else {
                Some(&mut *exceptfds)
            },
            timeout.as_mut(),
            sigmask
        )
        .map(|fds| fds
            .try_into()
            .expect("nfds is bound between [0, {FD_SETSIZE}]"))
        .or_minus_one_errno(),
        "pselect({}, {:p}, {:p}, {:p}, {:?} {})",
        nfds,
        readfds,
        writefds,
        exceptfds,
        timeout,
        sigmask
    )
}

// #[no_mangle]
// #[allow(unsafe_op_in_unsafe_fn)]
// pub unsafe fn pselect(
//     nfds: c_int,
//     readfds: *mut fd_set,
//     writefds: *mut fd_set,
//     exceptfds: *mut fd_set,
//     timeout: *const timespec,
//     sigmask: *const sigset_t,
// ) -> c_int {
//     let guard = tmp_disable_signals();
//
//     // Null pointers are valid for both pselect and sigprocmask.
//     // A null sigmask means pselect acts like select.
//     let mut saved_mask = 0;
//     if (!sigmask.is_null() && sigprocmask(SIG_SETMASK, sigmask, &mut saved_mask) == -1) {
//         return -1;
//     }
//
//     // select's timeout is a timeval whereas pselect's is a timespec.
//     // pselect doesn't modify the timeout with the remaining time so the result is ignored below
//     // and we don't need to convert timeval back to timespec.
//     let mut timeout: Option<timeval> = (!timeout.is_null()).then(|| (*timeout).into());
//
//     let result = select(
//         nfds,
//         readfds,
//         writefds,
//         exceptfds,
//         timeout.map_or(ptr::null_mut(), |mut t| &mut t),
//     );
//     if result == -1 && platform::ERRNO.get() == errno::EINTR {
//         manually_enter_trampoline();
//     }
//
//     if (!sigmask.is_null() && sigprocmask(SIG_SETMASK, &saved_mask, ptr::null_mut()) == -1) {
//         -1
//     } else {
//         result
//     }
// }
