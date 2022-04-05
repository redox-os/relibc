#![allow(non_snake_case)]

use alloc::{boxed::Box, collections::BTreeMap};
use core::{
    cell::UnsafeCell,
    intrinsics, ptr,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::{
    header::{sys_mman, time::timespec},
    ld_so::{
        linker::Linker,
        tcb::{Master, Tcb},
    },
    platform::{
        types::{c_int, c_uint, c_void, pid_t, size_t},
        Pal, Sys,
    },
    sync::{Mutex, Semaphore},
    ALLOCATOR,
};

type pte_osThreadHandle = pid_t;
type pte_osMutexHandle = *mut Mutex<()>;
type pte_osSemaphoreHandle = *mut Semaphore;
type pte_osThreadEntryPoint = unsafe extern "C" fn(params: *mut c_void) -> *mut c_void;

#[repr(C)]
#[derive(Eq, PartialEq)]
#[allow(dead_code)]
pub enum pte_osResult {
    PTE_OS_OK = 0,
    PTE_OS_NO_RESOURCES,
    PTE_OS_GENERAL_FAILURE,
    PTE_OS_TIMEOUT,
    PTE_OS_INTERRUPTED,
    PTE_OS_INVALID_PARAM,
}

use self::pte_osResult::*;

static mut pid_mutexes: Option<BTreeMap<pte_osThreadHandle, pte_osMutexHandle>> = None;
static mut pid_mutexes_lock: Mutex<()> = Mutex::new(());

static mut pid_stacks: Option<BTreeMap<pte_osThreadHandle, (*mut c_void, size_t)>> = None;
static mut pid_stacks_lock: Mutex<()> = Mutex::new(());

// TODO: VecMap/SLOB (speed) / radix tree (speed while allowing randomization for security).
#[thread_local]
static LOCALS: UnsafeCell<BTreeMap<c_uint, *mut c_void>> = UnsafeCell::new(BTreeMap::new());

static NEXT_KEY: AtomicU32 = AtomicU32::new(0);

unsafe fn locals<'a>() -> &'a mut BTreeMap<c_uint, *mut c_void> {
    &mut *LOCALS.get()
}

// pte_osResult pte_osInit(void)
#[no_mangle]
pub unsafe extern "C" fn pte_osInit() -> pte_osResult {
    PTE_OS_OK
}

/// A shim to wrap thread entry points in logic to set up TLS, for example
unsafe extern "C" fn pte_osThreadShim(
    entryPoint: pte_osThreadEntryPoint,
    argv: *mut c_void,
    mutex: pte_osMutexHandle,
    tls_size: usize,
    tls_masters_ptr: *mut Master,
    tls_masters_len: usize,
    tls_linker_ptr: *const Mutex<Linker>,
    tls_mspace: usize,
) {
    // The kernel allocated TLS does not have masters set, so do not attempt to copy it.
    // It will be copied by the kernel.
    if !tls_masters_ptr.is_null() {
        let tcb = Tcb::new(tls_size).unwrap();
        tcb.masters_ptr = tls_masters_ptr;
        tcb.masters_len = tls_masters_len;
        tcb.linker_ptr = tls_linker_ptr;
        tcb.mspace = tls_mspace;
        tcb.copy_masters().unwrap();
        tcb.activate();
    }

    // Wait until pte_osThreadStart
    pte_osMutexLock(mutex);
    entryPoint(argv);
    pte_osThreadExit();
}

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadCreate(
    entryPoint: pte_osThreadEntryPoint,
    stackSize: c_int,
    _initialPriority: c_int,
    argv: *mut c_void,
    ppte_osThreadHandle: *mut pte_osThreadHandle,
) -> pte_osResult {
    // Create a locked mutex, unlocked by pte_osThreadStart
    let mutex: pte_osMutexHandle = Box::into_raw(Box::new(Mutex::locked(())));

    let stack_size = if stackSize == 0 {
        1024 * 1024
    } else {
        stackSize as usize
    };
    let stack_base = sys_mman::mmap(
        ptr::null_mut(),
        stack_size,
        sys_mman::PROT_READ | sys_mman::PROT_WRITE,
        sys_mman::MAP_SHARED | sys_mman::MAP_ANONYMOUS,
        -1,
        0,
    );
    if stack_base as isize == -1 {
        return PTE_OS_GENERAL_FAILURE;
    }
    ptr::write_bytes(stack_base as *mut u8, 0, stack_size);
    let stack_end = stack_base.add(stack_size);
    let mut stack = stack_end as *mut usize;
    {
        let mut push = |value: usize| {
            stack = stack.offset(-1);
            *stack = value;
        };

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

        push(mutex as usize);

        push(argv as usize);
        push(entryPoint as usize);

        push(pte_osThreadShim as usize);
    }

    let id = Sys::pte_clone(stack);
    if id < 0 {
        return PTE_OS_GENERAL_FAILURE;
    }

    pte_osMutexLock(&mut pid_mutexes_lock);
    if pid_mutexes.is_none() {
        pid_mutexes = Some(BTreeMap::new());
    }
    pid_mutexes.as_mut().unwrap().insert(id, mutex);
    pte_osMutexUnlock(&mut pid_mutexes_lock);

    pte_osMutexLock(&mut pid_stacks_lock);
    if pid_stacks.is_none() {
        pid_stacks = Some(BTreeMap::new());
    }
    pid_stacks
        .as_mut()
        .unwrap()
        .insert(id, (stack_base, stack_size));
    pte_osMutexUnlock(&mut pid_stacks_lock);

    *ppte_osThreadHandle = id;

    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadStart(handle: pte_osThreadHandle) -> pte_osResult {
    let mut ret = PTE_OS_GENERAL_FAILURE;
    pte_osMutexLock(&mut pid_mutexes_lock);
    if let Some(ref mutexes) = pid_mutexes {
        if let Some(mutex) = mutexes.get(&handle) {
            pte_osMutexUnlock(*mutex);
            ret = PTE_OS_OK;
        }
    }
    pte_osMutexUnlock(&mut pid_mutexes_lock);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadExit() {
    Sys::exit(0);
}

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadExitAndDelete(handle: pte_osThreadHandle) -> pte_osResult {
    let res = pte_osThreadDelete(handle);
    if res != PTE_OS_OK {
        return res;
    }
    pte_osThreadExit();
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadDelete(handle: pte_osThreadHandle) -> pte_osResult {
    pte_osMutexLock(&mut pid_mutexes_lock);
    if let Some(ref mut mutexes) = pid_mutexes {
        if let Some(mutex) = mutexes.remove(&handle) {
            Box::from_raw(mutex);
        }
    }
    pte_osMutexUnlock(&mut pid_mutexes_lock);

    pte_osMutexLock(&mut pid_stacks_lock);
    if let Some(ref mut stacks) = pid_stacks {
        if let Some((stack_base, stack_size)) = stacks.remove(&handle) {
            //TODO: this currently unmaps the thread's stack, while it is being used!
            //sys_mman::munmap(stack_base, stack_size);
        }
    }
    pte_osMutexUnlock(&mut pid_stacks_lock);

    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadWaitForEnd(handle: pte_osThreadHandle) -> pte_osResult {
    let mut status = 0;
    Sys::waitpid(handle, &mut status, 0);
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadCancel(handle: pte_osThreadHandle) -> pte_osResult {
    //TODO: allow cancel of thread
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadCheckCancel(handle: pte_osThreadHandle) -> pte_osResult {
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadSleep(msecs: c_uint) {
    if msecs == 0 {
        Sys::sched_yield();
    } else {
        let tm = timespec {
            tv_sec: msecs as i64 / 1000,
            tv_nsec: (msecs % 1000) as i64 * 1000000,
        };
        Sys::nanosleep(&tm, ptr::null_mut());
    }
}

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadGetHandle() -> pte_osThreadHandle {
    Sys::gettid()
}

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadGetPriority(threadHandle: pte_osThreadHandle) -> c_int {
    // XXX Shouldn't Redox support priorities?
    1
}

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadSetPriority(
    threadHandle: pte_osThreadHandle,
    newPriority: c_int,
) -> pte_osResult {
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadGetMinPriority() -> c_int {
    1
}

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadGetMaxPriority() -> c_int {
    1
}

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadGetDefaultPriority() -> c_int {
    1
}

#[no_mangle]
pub unsafe extern "C" fn pte_osMutexCreate(pHandle: *mut pte_osMutexHandle) -> pte_osResult {
    *pHandle = Box::into_raw(Box::new(Mutex::new(())));
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osMutexDelete(handle: pte_osMutexHandle) -> pte_osResult {
    Box::from_raw(handle);
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osMutexLock(handle: pte_osMutexHandle) -> pte_osResult {
    (*handle).manual_lock();
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osMutexUnlock(handle: pte_osMutexHandle) -> pte_osResult {
    (*handle).manual_unlock();
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osSemaphoreCreate(
    initialValue: c_int,
    pHandle: *mut pte_osSemaphoreHandle,
) -> pte_osResult {
    *pHandle = Box::into_raw(Box::new(Semaphore::new(initialValue)));
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osSemaphoreDelete(handle: pte_osSemaphoreHandle) -> pte_osResult {
    Box::from_raw(handle);
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osSemaphorePost(
    handle: pte_osSemaphoreHandle,
    count: c_int,
) -> pte_osResult {
    (*handle).post();
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osSemaphorePend(
    handle: pte_osSemaphoreHandle,
    pTimeout: *mut c_uint,
) -> pte_osResult {
    let timeout_opt = if ! pTimeout.is_null() {
        let timeout = *pTimeout as i64;
        let tv_sec = timeout / 1000;
        let tv_nsec = (timeout % 1000) * 1000000;
        Some(timespec { tv_sec, tv_nsec })
    } else {
        None
    };
    (*handle).wait(timeout_opt.as_ref());
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osSemaphoreCancellablePend(
    handle: pte_osSemaphoreHandle,
    pTimeout: *mut c_uint,
) -> pte_osResult {
    //TODO: thread cancel
    pte_osSemaphorePend(handle, pTimeout)
}

#[no_mangle]
pub unsafe extern "C" fn pte_osAtomicExchange(ptarg: *mut c_int, val: c_int) -> c_int {
    intrinsics::atomic_xchg(ptarg, val)
}

#[no_mangle]
pub unsafe extern "C" fn pte_osAtomicCompareExchange(
    pdest: *mut c_int,
    exchange: c_int,
    comp: c_int,
) -> c_int {
    intrinsics::atomic_cxchg(pdest, comp, exchange).0
}

#[no_mangle]
pub unsafe extern "C" fn pte_osAtomicExchangeAdd(pAppend: *mut c_int, value: c_int) -> c_int {
    intrinsics::atomic_xadd(pAppend, value)
}

#[no_mangle]
pub unsafe extern "C" fn pte_osAtomicDecrement(pdest: *mut c_int) -> c_int {
    intrinsics::atomic_xadd(pdest, -1) - 1
}

#[no_mangle]
pub unsafe extern "C" fn pte_osAtomicIncrement(pdest: *mut c_int) -> c_int {
    intrinsics::atomic_xadd(pdest, 1) + 1
}

#[no_mangle]
pub unsafe extern "C" fn pte_osTlsSetValue(index: c_uint, value: *mut c_void) -> pte_osResult {
    locals().insert(index, value);
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osTlsGetValue(index: c_uint) -> *mut c_void {
    locals().get_mut(&index).copied().unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn pte_osTlsAlloc(pKey: *mut c_uint) -> pte_osResult {
    *pKey = NEXT_KEY.fetch_add(1, Ordering::Relaxed);
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osTlsFree(index: c_uint) -> pte_osResult {
    // XXX free keys
    PTE_OS_OK
}
