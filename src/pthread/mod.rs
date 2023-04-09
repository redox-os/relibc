//! Relibc Threads, or RLCT.

use core::cell::{Cell, UnsafeCell};
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::platform::{Pal, Sys, types::*};
use crate::header::sched::sched_param;
use crate::header::errno::*;
use crate::header::sys_mman;
use crate::header::pthread as header;
use crate::ld_so::{linker::Linker, tcb::{Master, Tcb}};
use crate::ALLOCATOR;

use crate::sync::{Mutex, waitval::Waitval};

const MAIN_PTHREAD_ID: usize = 1;

/// Called only by the main thread, as part of relibc_start.
pub unsafe fn init() {
    let obj = Box::into_raw(Box::new(Pthread {
        waitval: Waitval::new(),
        wants_cancel: AtomicBool::new(false),
        flags: PthreadFlags::empty().bits().into(),

        // TODO
        stack_base: core::ptr::null_mut(),
        stack_size: 0,

        os_tid: UnsafeCell::new(Sys::current_os_tid()),
    }));

    PTHREAD_SELF.set(obj);
}
pub unsafe fn terminate_from_main_thread() {
    for (tid, pthread) in OS_TID_TO_PTHREAD.lock().iter() {
        // TODO: Cancel?
        Sys::rlct_kill(*tid, crate::header::signal::SIGTERM);
    }
}

bitflags::bitflags! {
    struct PthreadFlags: usize {
        const DETACHED = 1;
    }
}

pub struct Pthread {
    waitval: Waitval<Retval>,
    wants_cancel: AtomicBool,
    flags: AtomicUsize,

    stack_base: *mut c_void,
    stack_size: usize,

    os_tid: UnsafeCell<OsTid>,
}

#[derive(Clone, Copy, Debug, Default, Ord, Eq, PartialOrd, PartialEq)]
pub struct OsTid {
    #[cfg(target_os = "redox")]
    pub context_id: usize,
    #[cfg(target_os = "linux")]
    pub thread_id: usize,
}

unsafe impl Send for Pthread {}
unsafe impl Sync for Pthread {}

use crate::header::bits_pthread::pthread_attr_t;

/// Positive error codes (EINVAL, not -EINVAL).
#[derive(Debug, Eq, PartialEq)]
// TODO: Move to a more generic place.
pub struct Errno(pub c_int);

#[derive(Clone, Copy)]
pub struct Retval(pub *mut c_void);

struct MmapGuard { page_start: *mut c_void, mmap_size: usize }
impl Drop for MmapGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = Sys::munmap(self.page_start, self.mmap_size);
        }
    }
}

pub unsafe fn create(attrs: Option<&pthread_attr_t>, start_routine: extern "C" fn(arg: *mut c_void) -> *mut c_void, arg: *mut c_void) -> Result<pthread_t, Errno> {
    let attrs = attrs.copied().unwrap_or_default();

    // Create a locked mutex, unlocked by the thread after it has started.
    let synchronization_mutex = Box::into_raw(Box::new(Mutex::locked(())));

    let stack_size = attrs.stacksize.next_multiple_of(Sys::getpagesize());

    // TODO: Custom stacks
    let stack_base = sys_mman::mmap(
        core::ptr::null_mut(),
        stack_size,
        sys_mman::PROT_READ | sys_mman::PROT_WRITE,
        sys_mman::MAP_SHARED | sys_mman::MAP_ANONYMOUS,
        -1,
        0,
    );
    if stack_base as isize == -1 {
        // "Insufficient resources"
        return Err(Errno(EAGAIN));
    }

    let mut flags = PthreadFlags::empty();
    match i32::from(attrs.detachstate) {
        header::PTHREAD_CREATE_DETACHED => flags |= PthreadFlags::DETACHED,
        header::PTHREAD_CREATE_JOINABLE => (),

        other => unreachable!("unknown detachstate {}", other),
    }

    let pthread = Pthread {
        waitval: Waitval::new(),
        flags: flags.bits().into(),
        wants_cancel: AtomicBool::new(false),
        stack_base,
        stack_size,
        os_tid: UnsafeCell::new(OsTid::default()),
    };
    let ptr = Box::into_raw(Box::new(pthread));

    let stack_raii = MmapGuard { page_start: stack_base, mmap_size: stack_size };

    let stack_end = stack_base.add(stack_size);
    let mut stack = stack_end as *mut usize;
    {
        let mut push = |value: usize| {
            stack = stack.sub(1);
            stack.write(value);
        };

        push(0);
        push(ptr as usize);

        //WARNING: Stack must be 128-bit aligned for SSE
        if let Some(tcb) = Tcb::current() {
            push(tcb.mspace as usize);
            push(tcb.linker_ptr as usize);
            push(tcb.masters_len);
            push(tcb.masters_ptr as usize);
            push(tcb.tls_len);
        } else {
            push(ALLOCATOR.get_book_keeper());
            push(0);
            push(0);
            push(0);
            push(0);
        }

        push(synchronization_mutex as usize);

        push(arg as usize);
        push(start_routine as usize);

        push(new_thread_shim as usize);
    }

    let Ok(os_tid) = Sys::rlct_clone(stack) else {
        return Err(Errno(EAGAIN));
    };
    core::mem::forget(stack_raii);

    let _ = (&*synchronization_mutex).lock();

    OS_TID_TO_PTHREAD.lock().insert(os_tid, ForceSendSync(ptr.cast()));

    Ok(ptr.cast())
}
/// A shim to wrap thread entry points in logic to set up TLS, for example
unsafe extern "C" fn new_thread_shim(
    entry_point: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    arg: *mut c_void,
    mutex: *const Mutex<()>,
    tls_size: usize,
    tls_masters_ptr: *mut Master,
    tls_masters_len: usize,
    tls_linker_ptr: *const Mutex<Linker>,
    tls_mspace: usize,
    pthread: *mut Pthread,
) -> ! {
    // TODO: Pass less arguments by allocating the TCB from the creator thread.
    if !tls_masters_ptr.is_null() {
        let tcb = Tcb::new(tls_size).unwrap();
        tcb.masters_ptr = tls_masters_ptr;
        tcb.masters_len = tls_masters_len;
        tcb.linker_ptr = tls_linker_ptr;
        tcb.mspace = tls_mspace;
        tcb.copy_masters().unwrap();
        tcb.activate();
    }
    PTHREAD_SELF.set(pthread);

    core::ptr::write((&*pthread).os_tid.get(), Sys::current_os_tid());

    (&*mutex).manual_unlock();
    let retval = entry_point(arg);

    exit_current_thread(Retval(retval))
}
pub unsafe fn join(thread: &Pthread) -> Result<Retval, Errno> {
    // We don't have to return EDEADLK, but unlike e.g. pthread_t lifetime checking, it's a
    // relatively easy check.
    if core::ptr::eq(thread, current_thread().unwrap_unchecked()) {
        return Err(Errno(EDEADLK));
    }

    // Waitval starts locked, and is unlocked when the thread finishes.
    let retval = *thread.waitval.wait();

    // We have now awaited the thread and received its return value. POSIX states that the
    // pthread_t of this thread, will no longer be valid. In practice, we can thus deallocate the
    // thread state.

    OS_TID_TO_PTHREAD.lock().remove(&thread.os_tid.get().read());

    dealloc_thread(thread);

    Ok(retval)
}

pub unsafe fn detach(thread: &Pthread) -> Result<(), Errno> {
    thread.flags.fetch_or(PthreadFlags::DETACHED.bits(), Ordering::Release);
    Ok(())
}

// Returns option because that's a no-op, but PTHREAD_SELF should always be initialized except in
// early init code.
pub fn current_thread() -> Option<&'static Pthread> {
    unsafe {
        NonNull::new(PTHREAD_SELF.get()).map(|p| p.as_ref())
    }
}

pub unsafe fn testcancel() {
    // TODO: Ordering
    if current_thread().unwrap_unchecked().wants_cancel.load(Ordering::SeqCst) {
        todo!("cancel")
    }
}

pub unsafe fn exit_current_thread(retval: Retval) -> ! {
    let this = current_thread().expect("failed to obtain current thread when exiting");

    if this.flags.load(Ordering::Acquire) & PthreadFlags::DETACHED.bits() != 0 {
        // When detached, the thread state no longer makes any sense, and can immediately be
        // deallocated.
        dealloc_thread(this);
    } else {
        // When joinable, the return value should be made available to other threads.
        this.waitval.post(retval);
    }

    Sys::exit_thread()
}

// TODO: Use Arc? One strong reference from each OS_TID_TO_PTHREAD and one strong reference from
// PTHREAD_SELF. The latter ref disappears when the thread exits, while the former disappears when
// detaching. Isn't that sufficient?
unsafe fn dealloc_thread(thread: &Pthread) {
    drop(Box::from_raw(thread as *const Pthread as *mut Pthread));
}
pub const SIGRT_RLCT_CANCEL: usize = 32;
pub const SIGRT_RLCT_TIMER: usize = 33;

unsafe fn cancel_sighandler(_: c_int) {
    // TODO: pthread_cleanup_push stack
    // TODO: thread-specific data

    // Terminate the thread
    exit_current_thread(Retval(header::PTHREAD_CANCELED));
}

pub unsafe fn cancel(thread: &Pthread) -> Result<(), Errno> {
    // TODO: Ordering
    thread.wants_cancel.store(true, Ordering::SeqCst);

    Sys::rlct_kill(thread.os_tid.get().read(), SIGRT_RLCT_CANCEL)?;

    todo!();
    Ok(())
}

// TODO: Hash map?
// TODO: RwLock
static OS_TID_TO_PTHREAD: Mutex<BTreeMap<OsTid, ForceSendSync<*mut Pthread>>> = Mutex::new(BTreeMap::new());

#[derive(Clone, Copy)]
struct ForceSendSync<T>(T);
unsafe impl<T> Send for ForceSendSync<T> {}
unsafe impl<T> Sync for ForceSendSync<T> {}

#[thread_local]
static PTHREAD_SELF: Cell<*mut Pthread> = Cell::new(core::ptr::null_mut());
