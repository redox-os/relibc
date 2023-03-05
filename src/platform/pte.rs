#![allow(non_snake_case)]

use alloc::{boxed::Box, collections::BTreeMap};
use core::{
    cell::UnsafeCell,
    intrinsics, ptr,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::{
    header::{
        sys_mman,
        time::{clock_gettime, timespec, CLOCK_MONOTONIC},
    },
    platform::{
        types::{c_int, c_long, c_uint, c_void, pid_t, size_t, time_t},
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
    if Sys::waitpid(handle, &mut status, 0) < 0 {
        return PTE_OS_GENERAL_FAILURE;
    }
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadCancel(handle: pte_osThreadHandle) -> pte_osResult {
    //TODO: allow cancel of thread
    eprintln!("pte_osThreadCancel");
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadCheckCancel(handle: pte_osThreadHandle) -> pte_osResult {
    //TODO: thread cancel
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadSleep(msecs: c_uint) {
    if msecs == 0 {
        Sys::sched_yield();
    } else {
        let tm = timespec {
            tv_sec: msecs as time_t / 1000,
            tv_nsec: ((msecs as c_long) % 1000) * 1000000,
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
    (*handle).post(count);
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osSemaphorePend(
    handle: pte_osSemaphoreHandle,
    pTimeout: *mut c_uint,
) -> pte_osResult {
    let timeout_opt = if !pTimeout.is_null() {
        // Get current time
        let mut time = timespec::default();
        clock_gettime(CLOCK_MONOTONIC, &mut time);

        // Add timeout to time
        let timeout = *pTimeout as time_t;
        time.tv_sec += timeout / 1000;
        time.tv_nsec += ((timeout % 1000) * 1_000_000) as c_long;
        while time.tv_nsec >= 1_000_000_000 {
            time.tv_sec += 1;
            time.tv_nsec -= 1_000_000_000;
        }
        Some(time)
    } else {
        None
    };
    match (*handle).wait(timeout_opt.as_ref()) {
        Ok(()) => PTE_OS_OK,
        Err(()) => PTE_OS_TIMEOUT,
    }
}
