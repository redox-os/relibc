use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::{mem, intrinsics, ptr};
use core::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};
use syscall;

use platform::types::{c_int, c_uint, c_void};

pub struct Semaphore {
    lock: i32,
    count: i32,
}

type pte_osThreadHandle = usize;
type pte_osMutexHandle = *mut i32;
type pte_osSemaphoreHandle = *mut Semaphore;
type pte_osThreadEntryPoint = unsafe extern "C" fn(params: *mut c_void) -> c_int;

#[repr(C)]
pub enum pte_osResult {
    PTE_OS_OK = 0,
    PTE_OS_NO_RESOURCES,
    PTE_OS_GENERAL_FAILURE,
    PTE_OS_TIMEOUT,
    PTE_OS_INTERRUPTED,
    PTE_OS_INVALID_PARAM
}

use self::pte_osResult::*;

static mut pid_mutexes: Option<BTreeMap<pte_osThreadHandle, pte_osMutexHandle>> = None;
static mut pid_mutexes_lock: i32 = 0;

#[thread_local]
static mut LOCALS: *mut BTreeMap<c_uint, *mut c_void> = ptr::null_mut();

static NEXT_KEY: AtomicUsize = ATOMIC_USIZE_INIT;

unsafe fn locals() -> &'static mut BTreeMap<c_uint, *mut c_void> {
    if LOCALS == ptr::null_mut() {
        LOCALS = Box::into_raw(Box::new(BTreeMap::new()));
    }
    &mut *LOCALS
}

// pte_osResult pte_osInit(void)
#[no_mangle]
pub unsafe extern "C" fn pte_osInit() -> pte_osResult {
    PTE_OS_OK
}

/*
pte_osResult pte_osThreadCreate(pte_osThreadEntryPoint entryPoint,
                                int stackSize,
                                int initialPriority,
                                void *argv,
                                pte_osThreadHandle* ppte_osThreadHandle)
*/

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadCreate(entryPoint: pte_osThreadEntryPoint,
                                   _stackSize: c_int,
                                   _initialPriority: c_int,
                                   argv: *mut c_void,
                                   ppte_osThreadHandle: *mut pte_osThreadHandle
                                   ) -> pte_osResult {
    // XXX error handling
    let id = syscall::clone(syscall::CLONE_VM | syscall::CLONE_FS | syscall::CLONE_FILES).unwrap();

    let mutex = Box::into_raw(Box::new(0));

    if id == 0 {
        // Wait until pte_osThreadStart
        pte_osMutexLock(mutex);
        entryPoint(argv);
        let _ = syscall::exit(0);
    } else {
        pte_osMutexLock(&mut pid_mutexes_lock);
        if pid_mutexes.is_none() {
            pid_mutexes = Some(BTreeMap::new());
        }
        pid_mutexes.as_mut().unwrap().insert(id, mutex);
        pte_osMutexUnlock(&mut pid_mutexes_lock);
        *ppte_osThreadHandle = id;
    }
    PTE_OS_OK
}

// pte_osResult pte_osThreadStart(pte_osThreadHandle osThreadHandle)
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

/*
void pte_osThreadExit()
pte_osResult pte_osThreadExitAndDelete(pte_osThreadHandle handle)
pte_osResult pte_osThreadDelete(pte_osThreadHandle handle)
*/
#[no_mangle]
pub unsafe extern "C" fn pte_osThreadExit() {
    syscall::exit(0);
}

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadExitAndDelete(handle: pte_osThreadHandle) -> pte_osResult {
    pte_osThreadDelete(handle);
    syscall::exit(0);
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
    PTE_OS_OK
}

// pte_osResult pte_osThreadWaitForEnd(pte_osThreadHandle threadHandle)
#[no_mangle]
pub unsafe extern "C" fn pte_osThreadWaitForEnd(handle: pte_osThreadHandle) -> pte_osResult {
    let mut status = 0;
    syscall::waitpid(handle, &mut status, 0);
    PTE_OS_OK
}

// pte_osResult pte_osThreadCancel(pte_osThreadHandle threadHandle)
#[no_mangle]
pub unsafe extern "C" fn pte_osThreadCancel(handle: pte_osThreadHandle) -> pte_osResult {
    //TODO: allow cancel of thread
    PTE_OS_OK
}

// pte_osResult pte_osThreadCheckCancel(pte_osThreadHandle threadHandle)
#[no_mangle]
pub unsafe extern "C" fn pte_osThreadCheckCancel(handle: pte_osThreadHandle) -> pte_osResult {
    PTE_OS_OK
}

//void pte_osThreadSleep(unsigned int msecs)
#[no_mangle]
pub unsafe extern "C" fn pte_osThreadSleep(msecs: c_uint) {
    let tm = syscall::TimeSpec {
        tv_sec: msecs as i64 / 1000,
        tv_nsec: (msecs % 1000) as i32 * 1000000,
    };
    let mut rmtp = mem::uninitialized();
    let _ = syscall::nanosleep(&tm, &mut rmtp);
}

// pte_osThreadHandle pte_osThreadGetHandle(void)
#[no_mangle]
pub unsafe extern "C" fn pte_osThreadGetHandle() -> pte_osThreadHandle {
    syscall::getpid().unwrap()
}

/*
int pte_osThreadGetPriority(pte_osThreadHandle threadHandle)
pte_osResult pte_osThreadSetPriority(pte_osThreadHandle threadHandle, int newPriority)
int pte_osThreadGetMinPriority()
int pte_osThreadGetMaxPriority()
int pte_osThreadGetDefaultPriority()
*/

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadGetPriority(threadHandle: pte_osThreadHandle) -> c_int {
    // XXX Shouldn't Redox support priorities?
    1
}

#[no_mangle]
pub unsafe extern "C" fn pte_osThreadSetPriority(threadHandle: pte_osThreadHandle, newPriority: c_int) -> pte_osResult {
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

/*
pte_osResult pte_osMutexCreate(pte_osMutexHandle *pHandle)
pte_osResult pte_osMutexDelete(pte_osMutexHandle handle)
pte_osResult pte_osMutexLock(pte_osMutexHandle handle)
pte_osResult pte_osMutexUnlock(pte_osMutexHandle handle)
*/

#[no_mangle]
pub unsafe extern "C" fn pte_osMutexCreate(pHandle: *mut pte_osMutexHandle) -> pte_osResult {
    *pHandle = Box::into_raw(Box::new(0));
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osMutexDelete(handle: pte_osMutexHandle) -> pte_osResult {
    Box::from_raw(handle);
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osMutexLock(handle: pte_osMutexHandle) -> pte_osResult {
    let mut c = 0;
    for _i in 0..100 {
        c = intrinsics::atomic_cxchg(handle, 0, 1).0;
        if c == 0 {
            break;
        }
    }
    if c == 1 {
        c = intrinsics::atomic_xchg(handle, 2);
    }
    while c != 0 {
        let _ = syscall::futex(handle, syscall::FUTEX_WAIT, 2, 0, ptr::null_mut());
        c = intrinsics::atomic_xchg(handle, 2);
    }

    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osMutexUnlock(handle: pte_osMutexHandle) -> pte_osResult {
    if *handle == 2 {
        *handle = 0;
    } else if intrinsics::atomic_xchg(handle, 0) == 1 {
        return PTE_OS_OK;
    }
    for _i in 0..100 {
        if *handle != 0 {
            if intrinsics::atomic_cxchg(handle, 1, 2).0 != 0 {
                return PTE_OS_OK;
            }
        }
    }
    let _ = syscall::futex(handle, syscall::FUTEX_WAKE, 1, 0, ptr::null_mut());

    PTE_OS_OK
}

/*
pte_osResult pte_osSemaphoreCreate(int initialValue, pte_osSemaphoreHandle *pHandle)
pte_osResult pte_osSemaphoreDelete(pte_osSemaphoreHandle handle)
pte_osResult pte_osSemaphorePost(pte_osSemaphoreHandle handle, int count)
pte_osResult pte_osSemaphorePend(pte_osSemaphoreHandle handle, unsigned int *pTimeoutMsecs)
pte_osResult pte_osSemaphoreCancellablePend(pte_osSemaphoreHandle semHandle, unsigned int *pTimeout)
*/

#[no_mangle]
pub unsafe extern "C" fn pte_osSemaphoreCreate(initialValue: c_int, pHandle: *mut pte_osSemaphoreHandle) -> pte_osResult {
    *pHandle = Box::into_raw(Box::new(Semaphore {
        lock: 0,
        count: initialValue,
    }));
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osSemaphoreDelete(handle: pte_osSemaphoreHandle) -> pte_osResult {
    Box::from_raw(handle);
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osSemaphorePost(handle: pte_osSemaphoreHandle, count: c_int) -> pte_osResult {
    let semaphore = &mut *handle;
    pte_osMutexLock(&mut semaphore.lock);
    intrinsics::atomic_xadd(&mut semaphore.count, 1);
    pte_osMutexUnlock(&mut semaphore.lock);
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osSemaphorePend(handle: pte_osSemaphoreHandle, pTimeout: *mut c_uint) -> pte_osResult {
    //TODO: pTimeout
    let semaphore = &mut *handle;
    let mut acquired = false;
    while ! acquired {
        pte_osMutexLock(&mut semaphore.lock);
        if intrinsics::atomic_load(&mut semaphore.count) > 0 {
            intrinsics::atomic_xsub(&mut semaphore.count, 1);
            acquired = true;
        }
        pte_osMutexUnlock(&mut semaphore.lock);
        let _ = syscall::sched_yield();
    }
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osSemaphoreCancellablePend(handle: pte_osSemaphoreHandle, pTimeout: *mut c_uint) -> pte_osResult {
    //TODO
    pte_osSemaphorePend(handle, pTimeout)
}

/*
int pte_osAtomicExchange(int *ptarg, int val)
int pte_osAtomicCompareExchange(int *pdest, int exchange, int comp)
int pte_osAtomicExchangeAdd(int volatile* pAddend, int value)
int pte_osAtomicDecrement(int *pdest)
int pte_osAtomicIncrement(int *pdest)
*/

#[no_mangle]
pub unsafe extern "C" fn pte_osAtomicExchange(ptarg: *mut c_int, val: c_int) -> c_int {
    intrinsics::atomic_xchg(ptarg, val)
}

#[no_mangle]
pub unsafe extern "C" fn pte_osAtomicCompareExchange(pdest: *mut c_int, exchange: c_int, comp: c_int) -> c_int {
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

/*
pte_osResult pte_osTlsSetValue(unsigned int index, void * value)
void * pte_osTlsGetValue(unsigned int index)
pte_osResult pte_osTlsAlloc(unsigned int *pKey)
pte_osResult pte_osTlsFree(unsigned int index)
*/

#[no_mangle]
pub unsafe extern "C" fn pte_osTlsSetValue(index: c_uint, value: *mut c_void) -> pte_osResult {
    locals().insert(index, value);
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osTlsGetValue(index: c_uint) -> *mut c_void {
    locals().get_mut(&index).map(|x| *x).unwrap_or(ptr::null_mut())
}

#[no_mangle]
pub unsafe extern "C" fn pte_osTlsAlloc(pKey: *mut c_uint) -> pte_osResult {
    NEXT_KEY.fetch_add(1, Ordering::SeqCst);
    PTE_OS_OK
}

#[no_mangle]
pub unsafe extern "C" fn pte_osTlsFree(index: c_uint) -> pte_osResult {
    // XXX free keys
    PTE_OS_OK
}
