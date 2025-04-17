//! Relibc Threads, or RLCT.

use core::{
    cell::{Cell, UnsafeCell},
    mem::{offset_of, MaybeUninit},
    ptr::{addr_of, NonNull},
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use alloc::{boxed::Box, collections::BTreeMap};

use crate::{
    error::Errno,
    header::{errno::*, pthread as header, sched::sched_param, sys_mman},
    ld_so::{
        linker::Linker,
        tcb::{Master, Tcb},
        ExpectTlsFree,
    },
    platform::{types::*, Pal, Sys},
};

use crate::sync::{waitval::Waitval, Mutex};

/// Called only by the main thread, as part of relibc_start.
pub unsafe fn init() {
    Tcb::current()
        .expect_notls("no TCB present for main thread")
        .pthread = Pthread {
        waitval: Waitval::new(),
        has_enabled_cancelation: AtomicBool::new(false),
        has_queued_cancelation: AtomicBool::new(false),
        flags: PthreadFlags::empty().bits().into(),

        //index: FIRST_THREAD_IDX,

        // TODO
        stack_base: core::ptr::null_mut(),
        stack_size: 0,

        os_tid: UnsafeCell::new(Sys::current_os_tid()),
    };
}

//static NEXT_INDEX: AtomicU32 = AtomicU32::new(FIRST_THREAD_IDX + 1);
//const FIRST_THREAD_IDX: usize = 1;

pub unsafe fn terminate_from_main_thread() {
    for (_, tcb) in OS_TID_TO_PTHREAD.lock().iter() {
        let _ = cancel(&(*tcb.0).pthread);
    }
}

bitflags::bitflags! {
    struct PthreadFlags: usize {
        const DETACHED = 1;
    }
}

#[derive(Debug)]
pub struct Pthread {
    pub(crate) waitval: Waitval<Retval>,
    pub(crate) has_queued_cancelation: AtomicBool,
    pub(crate) has_enabled_cancelation: AtomicBool,
    pub(crate) flags: AtomicUsize,

    pub(crate) stack_base: *mut c_void,
    pub(crate) stack_size: usize,

    pub os_tid: UnsafeCell<OsTid>,
}

#[derive(Clone, Copy, Debug, Default, Ord, Eq, PartialOrd, PartialEq)]
pub struct OsTid {
    #[cfg(target_os = "redox")]
    pub thread_fd: usize,
    #[cfg(target_os = "linux")]
    pub thread_id: usize,
}

unsafe impl Send for Pthread {}
unsafe impl Sync for Pthread {}

#[derive(Clone, Copy, Debug)]
pub struct Retval(pub *mut c_void);

struct MmapGuard {
    page_start: *mut c_void,
    mmap_size: usize,
}
impl Drop for MmapGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = Sys::munmap(self.page_start, self.mmap_size);
        }
    }
}

pub(crate) unsafe fn create(
    attrs: Option<&header::RlctAttr>,
    start_routine: extern "C" fn(arg: *mut c_void) -> *mut c_void,
    arg: *mut c_void,
) -> Result<pthread_t, Errno> {
    let attrs = attrs.copied().unwrap_or_default();

    let mut current_sigmask = 0_u64;
    #[cfg(target_os = "redox")]
    {
        current_sigmask =
            redox_rt::signal::get_sigmask().expect("failed to obtain sigprocmask for caller");
    }

    // Create a locked mutex, unlocked by the thread after it has started.
    let synchronization_mutex = Mutex::locked(current_sigmask);
    let synchronization_mutex = &synchronization_mutex;

    let tid_mutex = Mutex::<MaybeUninit<OsTid>>::new(MaybeUninit::uninit());
    let mut tid_guard = tid_mutex.lock();

    let stack_size = attrs.stacksize.next_multiple_of(Sys::getpagesize());

    let stack_base = if attrs.stack != 0 {
        attrs.stack as *mut c_void
    } else {
        let ret = sys_mman::mmap(
            core::ptr::null_mut(),
            stack_size,
            sys_mman::PROT_READ | sys_mman::PROT_WRITE,
            sys_mman::MAP_PRIVATE | sys_mman::MAP_ANONYMOUS,
            -1,
            0,
        );
        if ret as isize == -1 {
            // "Insufficient resources"
            return Err(Errno(EAGAIN));
        }
        ret
    };

    let mut flags = PthreadFlags::empty();
    match i32::from(attrs.detachstate) {
        header::PTHREAD_CREATE_DETACHED => flags |= PthreadFlags::DETACHED,
        header::PTHREAD_CREATE_JOINABLE => (),

        other => unreachable!("unknown detachstate {}", other),
    }

    let stack_raii = MmapGuard {
        page_start: stack_base,
        mmap_size: stack_size,
    };

    let current_tcb = Tcb::current().expect("no TCB!");
    let new_tcb = Tcb::new(current_tcb.tls_len).map_err(|_| Errno(ENOMEM))?;
    new_tcb.pthread.flags = flags.bits().into();
    new_tcb.pthread.stack_base = stack_base;
    new_tcb.pthread.stack_size = stack_size;

    new_tcb.masters_ptr = current_tcb.masters_ptr;
    new_tcb.masters_len = current_tcb.masters_len;
    new_tcb.linker_ptr = current_tcb.linker_ptr;
    new_tcb.mspace = current_tcb.mspace;

    let stack_end = stack_base.add(stack_size);
    let mut stack = stack_end as *mut usize;
    {
        let mut push = |value: usize| {
            stack = stack.sub(1);
            stack.write(value);
        };

        if cfg!(target_arch = "aarch64") {
            // Aarch64 requires the stack to be 16 byte aligned after
            // the call instruction, unlike x86 which requires it to be
            // aligned before the call instruction. As such push an
            // extra word on the stack to align the stack to 16 bytes.
            push(0);
        }
        push(0);
        push(synchronization_mutex as *const _ as usize);
        push(addr_of!(tid_mutex) as usize);
        push(new_tcb as *mut _ as usize);

        push(arg as usize);
        push(start_routine as usize);

        push(new_thread_shim as usize);
    }

    let Ok(os_tid) = Sys::rlct_clone(stack) else {
        return Err(Errno(EAGAIN));
    };
    core::mem::forget(stack_raii);

    tid_guard.write(os_tid);
    drop(tid_guard);
    let _ = synchronization_mutex.lock();

    OS_TID_TO_PTHREAD
        .lock()
        .insert(os_tid, ForceSendSync(new_tcb));

    Ok((&new_tcb.pthread) as *const _ as *mut _)
}
/// A shim to wrap thread entry points in logic to set up TLS, for example
unsafe extern "C" fn new_thread_shim(
    entry_point: unsafe extern "C" fn(*mut c_void) -> *mut c_void,
    arg: *mut c_void,
    tcb: *mut Tcb,
    mutex1: *const Mutex<MaybeUninit<OsTid>>,
    mutex2: *const Mutex<u64>,
) -> ! {
    let tid = (*(&*mutex1).lock()).assume_init();

    if let Some(tcb) = tcb.as_mut() {
        #[cfg(not(target_os = "redox"))]
        {
            tcb.activate();
        }
        #[cfg(target_os = "redox")]
        {
            tcb.activate(redox_rt::proc::FdGuard::new(tid.thread_fd));
            redox_rt::signal::setup_sighandler(&tcb.os_specific, false);
        }
    }

    let procmask = (&*mutex2).as_ptr().read();

    if let Some(tcb) = tcb.as_mut() {
        tcb.copy_masters().unwrap();
    }

    (*tcb).pthread.os_tid.get().write(Sys::current_os_tid());

    (&*mutex2).manual_unlock();

    #[cfg(target_os = "redox")]
    {
        redox_rt::signal::set_sigmask(Some(procmask), None)
            .expect("failed to set procmask in child thread");
    }

    let retval = entry_point(arg);

    exit_current_thread(Retval(retval))
}
pub unsafe fn join(thread: &Pthread) -> Result<Retval, Errno> {
    // We don't have to return EDEADLK, but unlike e.g. pthread_t lifetime checking, it's a
    // relatively easy check.
    if core::ptr::eq(
        thread,
        current_thread().expect("current thread not present"),
    ) {
        return Err(Errno(EDEADLK));
    }

    // Waitval starts locked, and is unlocked when the thread finishes.
    let retval = *thread.waitval.wait();

    // We have now awaited the thread and received its return value. POSIX states that the
    // pthread_t of this thread, will no longer be valid. In practice, we can thus deallocate the
    // thread state.

    dealloc_thread(thread);

    Ok(retval)
}

pub unsafe fn detach(thread: &Pthread) -> Result<(), Errno> {
    thread
        .flags
        .fetch_or(PthreadFlags::DETACHED.bits(), Ordering::AcqRel);
    Ok(())
}

pub fn current_thread() -> Option<&'static Pthread> {
    unsafe { Tcb::current().map(|p| &p.pthread) }
}

pub unsafe fn testcancel() {
    let this_thread = current_thread().expect("current thread not present");

    if this_thread.has_queued_cancelation.load(Ordering::Acquire)
        && this_thread.has_enabled_cancelation.load(Ordering::Acquire)
    {
        cancel_current_thread();
    }
}

pub unsafe fn exit_current_thread(retval: Retval) -> ! {
    // Run pthread_cleanup_push/pthread_cleanup_pop destructors.
    header::run_destructor_stack();

    header::tls::run_all_destructors();

    let this = current_thread().expect("failed to obtain current thread when exiting");
    let stack_base = this.stack_base;
    let stack_size = this.stack_size;

    if this.flags.load(Ordering::Acquire) & PthreadFlags::DETACHED.bits() != 0 {
        // When detached, the thread state no longer makes any sense, and can immediately be
        // deallocated.
        dealloc_thread(this);
    } else {
        // When joinable, the return value should be made available to other threads.
        this.waitval.post(retval);
    }

    Sys::exit_thread(stack_base.cast(), stack_size)
}

unsafe fn dealloc_thread(thread: &Pthread) {
    // TODO: How should this be handled on Linux?
    OS_TID_TO_PTHREAD.lock().remove(&thread.os_tid.get().read());
}
pub const SIGRT_RLCT_CANCEL: usize = 33;
pub const SIGRT_RLCT_TIMER: usize = 34;

unsafe extern "C" fn cancel_sighandler(_: c_int) {
    cancel_current_thread();
}
unsafe fn cancel_current_thread() {
    // Terminate the thread
    exit_current_thread(Retval(header::PTHREAD_CANCELED));
}

pub unsafe fn cancel(thread: &Pthread) -> Result<(), Errno> {
    // TODO: What order should these atomic bools be accessed in?
    thread.has_queued_cancelation.store(true, Ordering::Release);

    if thread.has_enabled_cancelation.load(Ordering::Acquire) {
        Sys::rlct_kill(thread.os_tid.get().read(), SIGRT_RLCT_CANCEL)?;
    }

    Ok(())
}

pub fn set_sched_param(
    _thread: &Pthread,
    _policy: c_int,
    _param: &sched_param,
) -> Result<(), Errno> {
    // TODO
    Ok(())
}
pub fn set_sched_priority(_thread: &Pthread, _prio: c_int) -> Result<(), Errno> {
    // TODO
    Ok(())
}
pub fn set_cancel_state(state: c_int) -> Result<c_int, Errno> {
    let this_thread = current_thread().expect("current thread not present");

    let was_cancelable = match state {
        header::PTHREAD_CANCEL_ENABLE => {
            let old = this_thread
                .has_enabled_cancelation
                .swap(true, Ordering::Release);

            if this_thread.has_queued_cancelation.load(Ordering::Acquire) {
                unsafe {
                    cancel_current_thread();
                }
            }
            old
        }
        header::PTHREAD_CANCEL_DISABLE => this_thread
            .has_enabled_cancelation
            .swap(false, Ordering::Release),

        _ => return Err(Errno(EINVAL)),
    };

    Ok(match was_cancelable {
        true => header::PTHREAD_CANCEL_ENABLE,
        false => header::PTHREAD_CANCEL_DISABLE,
    })
}
pub fn set_cancel_type(ty: c_int) -> Result<c_int, Errno> {
    let this_thread = current_thread().expect("current thread not present");

    // TODO
    match ty {
        header::PTHREAD_CANCEL_DEFERRED => (),
        header::PTHREAD_CANCEL_ASYNCHRONOUS => (),

        _ => return Err(Errno(EINVAL)),
    }
    Ok(header::PTHREAD_CANCEL_DEFERRED)
}
pub fn get_cpu_clkid(thread: &Pthread) -> Result<clockid_t, Errno> {
    // TODO
    Err(Errno(ENOENT))
}
pub fn get_sched_param(thread: &Pthread) -> Result<(clockid_t, sched_param), Errno> {
    todo!()
}

// TODO: Hash map?
// TODO: RwLock to improve perf?
static OS_TID_TO_PTHREAD: Mutex<BTreeMap<OsTid, ForceSendSync<*mut Tcb>>> =
    Mutex::new(BTreeMap::new());

#[derive(Clone, Copy)]
struct ForceSendSync<T>(T);
unsafe impl<T> Send for ForceSendSync<T> {}
unsafe impl<T> Sync for ForceSendSync<T> {}

/*pub(crate) fn current_thread_index() -> u32 {
    current_thread().expect("current thread not present").index
}*/

#[derive(Clone, Copy, Default, Debug)]
pub enum Pshared {
    #[default]
    Private,

    Shared,
}
impl Pshared {
    pub const fn from_raw(raw: c_int) -> Option<Self> {
        Some(match raw {
            header::PTHREAD_PROCESS_PRIVATE => Self::Private,
            header::PTHREAD_PROCESS_SHARED => Self::Shared,

            _ => return None,
        })
    }
    pub const fn raw(self) -> c_int {
        match self {
            Self::Private => header::PTHREAD_PROCESS_PRIVATE,
            Self::Shared => header::PTHREAD_PROCESS_SHARED,
        }
    }
}
