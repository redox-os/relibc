use core::cell::{Cell, UnsafeCell};
use core::ptr::NonNull;
use core::sync::atomic::{AtomicBool, Ordering};

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::platform::{Pal, Sys, types::*};
use crate::header::sched::sched_param;
use crate::header::errno::*;
use crate::header::sys_mman;
use crate::ld_so::{linker::Linker, tcb::{Master, Tcb}};

use crate::sync::{Mutex, waitval::Waitval};

extern "C" fn pthread_init() {
}
extern "C" fn pthread_terminate() {
}

struct Pthread {
    waitval: Waitval<Retval>,
    wants_cancel: AtomicBool,

    stack_base: *mut c_void,
    stack_size: usize,

    os_tid: OsTid,
}

#[derive(Clone, Copy, Debug, Ord, Eq, PartialOrd, PartialEq)]
pub struct OsTid {
    #[cfg(target_os = "redox")]
    context_id: usize,
    #[cfg(target_os = "linux")]
    thread_id: usize,
}

unsafe impl Send for Pthread {}
unsafe impl Sync for Pthread {}

use crate::header::pthread::attr::Attr;

/// Positive error codes (EINVAL, not -EINVAL).
#[derive(Debug)]
pub struct Errno(pub c_int);

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
        attrs.stacksize,
        sys_mman::PROT_READ | sys_mman::PROT_WRITE,
        sys_mman::MAP_SHARED | sys_mman::MAP_ANONYMOUS,
        -1,
        0,
    );
    if stack_base as isize == -1 {
        // "Insufficient resources"
        return Err(Errno(EAGAIN));
    }

    let pthread = Pthread {
        waitval: Waitval::new(),
        wants_cancel: AtomicBool::new(false),
        stack_base,
        stack_size,
        os_tid,
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

    let id = Sys::pte_clone(stack);
    if id < 0 {
        return Err(Errno(EAGAIN));
    }

    let _ = (&mut *synchronization_mutex).lock();
    CID_TO_PTHREAD.lock().insert(id, ForceSendSync(ptr.cast()));

    core::mem::forget(stack_raii);

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

    // TODO: Deinitialization code

    Ok(retval)
}

pub unsafe fn detach(thread: &Pthread) -> Result<(), Errno> {
    todo!()
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
    let this = current_thread().unwrap_unchecked();
    this.waitval.post(retval);

    Sys::exit_thread()
}

pub unsafe fn cancel(thread: &Pthread) -> Result<(), Errno> {
    thread.wants_cancel.store(true, Ordering::SeqCst);
    crate::header::signal
}

// TODO: Hash map?
static OS_TID_TO_PTHREAD: Mutex<BTreeMap<OsTid, ForceSendSync<pthread_t>>> = Mutex::new(BTreeMap::new());

struct ForceSendSync<T>(T);
unsafe impl<T> Send for ForceSendSync<T> {}
unsafe impl<T> Sync for ForceSendSync<T> {}

#[thread_local]
static PTHREAD_SELF: Cell<*mut Pthread> = Cell::new(core::ptr::null_mut());

pub struct ForkHandlers {
    pub prepare: Vec<extern "C" fn()>,
    pub parent: Vec<extern "C" fn()>,
    pub child: Vec<extern "C" fn()>,
}

// TODO: Use fork handlers
// TODO: Append-only atomic queue, not because of performance, but because of atomicity.
pub static FORK_HANDLERS: Mutex<ForkHandlers> = Mutex::new(ForkHandlers {
    parent: Vec::new(),
    child: Vec::new(),
    prepare: Vec::new(),
});
